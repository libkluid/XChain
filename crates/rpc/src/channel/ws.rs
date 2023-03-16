use futures::lock::Mutex;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};

use crate::jsonrpc::{JsonRpc, Response};
use crate::channel::OneshotChannel;
use crate::Error;

type Session = WebSocketStream<MaybeTlsStream<TcpStream>>;

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
}

pub struct WebsocketOneshotChannel {
    session: Mutex<Option<Session>>,
    endpoint: String,
}

impl WebsocketOneshotChannel {
    async fn connection(&self) -> Result<Session, Error> {
        log::info!("Connecting to {}", self.endpoint.as_str());
        let (stream, _) = tokio_tungstenite::connect_async(self.endpoint.clone()).await?;
        Ok(stream)
    }
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
                *session_lock = Some(self.connection().await?);
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