use std::pin::Pin;
use std::task::{Context, Poll};
use futures::lock::Mutex;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};

use crate::jsonrpc::{JsonRpc, Response};
use crate::channel::{OneshotChannel, SubscriptionChannel};
use crate::Error;

use super::Subscriber;

type Session = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn connection(endpoint: &str) -> Result<Session, Error> {
    let (stream, _) = tokio_tungstenite::connect_async(endpoint).await?;
    Ok(stream)
}

pub struct WebsocketChannel;

impl WebsocketChannel {
    pub fn oneshot<E>(endpoint: E) -> WebsocketOneshotChannel
    where
        E: Into<String>,
    {
        WebsocketOneshotChannel {
            session: Mutex::new(None),
            endpoint: endpoint.into(),
        }
    }

    pub fn subscription<E>(endpoint: E) -> WebsocketSubscriptionChannel
    where
        E: Into<String>,
    {
        WebsocketSubscriptionChannel {
            endpoint: endpoint.into(),
        }
    }
}

pub struct WebsocketOneshotChannel {
    session: Mutex<Option<Session>>,
    endpoint: String,
}


#[async_trait]
impl OneshotChannel for WebsocketOneshotChannel {
    type Output = Response;

    // TODO: Handle reconnect
    async fn fire(&self, json: &JsonRpc) -> Result<Self::Output, Error> {
        let mut session_lock = self.session.lock().await;
        let session = match session_lock.as_mut() {
            Some(session) => session,
            None => {
                log::info!("Connecting to {}", self.endpoint.as_str());
                *session_lock = Some(connection(self.endpoint.as_str()).await?);
                match session_lock.as_mut() {
                    Some(session) => session,
                    _ => Err(Error::ConnectionError(self.endpoint.clone()))?
                }
            }
        };

        let message = match serde_json::to_string(json) {
            Ok(message) => message,
            Err(err) => Err(Error::JsonformatError(err.into()))?
        };

        let send_message = tungstenite::Message::Text(message);
        session.send(send_message).await?;

        let response = session.next().await.ok_or(Error::ConnectionError(self.endpoint.clone()))?;
        let recv_message = response?;

        let response = match recv_message {
            tungstenite::Message::Text(text) => {
                match serde_json::from_str::<Response>(&text) {
                    Ok(response) => response,
                    Err(err) => Err(Error::JsonformatError(err.into()))?
                }
            },
            _ => Err(Error::ConnectionError(self.endpoint.clone()))?
        };

        Ok(response)
    }
}

pub struct WebsocketSubscriptionChannel {
    endpoint: String,
}

impl WebsocketSubscriptionChannel {
    async fn connection(&self) -> Result<Session, Error> {
        log::info!("Connecting to {}", self.endpoint.as_str());
        let (stream, _) = tokio_tungstenite::connect_async(self.endpoint.clone()).await?;
        Ok(stream)
    }
}

#[pin_project]
pub struct WebsocketSubscription {
    #[pin]
    stream: futures::stream::SplitStream<Session>,
}

impl WebsocketSubscription {
    fn new(stream: futures::stream::SplitStream<Session>) -> Self {
        Self { stream }
    }
}

impl futures::Stream for WebsocketSubscription {
    type Item = Result<JsonRpc, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let this = self.as_mut().project();
            match this.stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(Ok(tungstenite::Message::Text(text)))) => {
                    match serde_json::from_str::<JsonRpc>(&text) {
                        Ok(response) => return Poll::Ready(Some(Ok(response))),
                        Err(err) => {
                            log::error!("Failed to parse response: {}", err);
                            return Err(Error::JsonformatError(err.into()))?
                        }
                    }
                },
                Poll::Ready(Some(Ok(tungstenite::Message::Binary(bytes)))) => {
                    match serde_json::from_slice::<JsonRpc>(bytes.as_ref()) {
                        Ok(response) => return Poll::Ready(Some(Ok(response))),
                        Err(err) => {
                            log::error!("Failed to parse response: {}", err);
                            return Err(Error::JsonformatError(err.into()))?
                        }
                    }
                },
                Poll::Ready(Some(Ok(tungstenite::Message::Pong(_)))) => {
                    log::debug!("Received Pong");
                    continue
                }
                rest => {
                    unimplemented!("WebsocketSubscription received unsupported messsage: {:?}", rest)
                }
            }

        }
    }
}

#[async_trait]
impl SubscriptionChannel for WebsocketSubscriptionChannel {
    type Item = JsonRpc;

    async fn subscribe(&self, jsonrpc: &JsonRpc) -> Result<Subscriber<Self::Item>, Error> {
        let session = self.connection().await?;
        let (mut writer, mut reader) = session.split();

        let message = match serde_json::to_string(jsonrpc) {
            Ok(message) => message,
            Err(err) => Err(Error::JsonformatError(err.into()))?
        };

        let send_message = tungstenite::Message::Text(message);
        writer.send(send_message).await?;
        let _response = reader.next().await.ok_or(Error::ResponseDroppedError)?;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                let message = tungstenite::Message::Ping(Vec::new());
                writer.send(message).await.ok();
            }
        });

        let websocket_subscription = WebsocketSubscription::new(reader);
        let subscriber = Subscriber::new(Box::new(websocket_subscription));
        Ok(subscriber)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonrpc::{JsonRpc, Id};
    use crate::channel::OneshotChannel;

    #[tokio::test]
    async fn test_websocket_channel() {
        let channel = WebsocketChannel::oneshot("wss://ws.wemix.com");
        let jsonrpc = JsonRpc::format(
            Id::Num(1),
            "eth_blockNumber",
            json!(null),
        );

        let response = channel.fire(&jsonrpc).await.expect("Failed to send request");
        assert_eq!(response.id, Id::Num(1));
    }
}
