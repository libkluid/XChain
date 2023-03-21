use std::rc::Rc;
use contracts::eth::{EthereumContract, EthereumFunction};
use ethabi::num_traits::ToPrimitive;
use ethabi::{Value, num_bigint::BigUint};
use rpc::jsonrpc::Tag;
use rpc::channel::HttpChannel;
use rpc::network::{EthereumNetwork, NetworkOptions};

const FERRO_2FER_SWAP: &'static str = "0xa34C0fE36541fB085677c36B4ff0CCF5fa2B32d6";
// const SLACK_WEBHOOK_URL: &'static str = "https://hooks.slack.com/services/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";
const SLACK_WEBHOOK_URL: &'static str = "https://hooks.slack.com/services/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";

fn decimalize(balance: &BigUint, decimals: u8) -> String {
    let scale = BigUint::from(10_u32).pow(decimals.into());
    let significand = format!("{}", balance / &scale);
    let mantissa = format!("{}", balance % &scale);
    format!("{}.{}", significand, &mantissa[..2])
}

#[tokio::main]
async fn main() {
    let channel = Rc::new(HttpChannel::new("https://evm.cronos.org"));
    let network = Rc::new(EthereumNetwork::new(NetworkOptions { radix: 16 }));

    let contract = EthereumContract::new(network, channel, FERRO_2FER_SWAP);
    let function = EthereumFunction::new("getTokenBalance", &["uint8"], &["uint256"]).unwrap();

    let results = contract.invoke(&function, vec![Value::UInt(BigUint::from(0_u8))], Tag::Latest).await.unwrap();
    let balance_usdc = results[0].as_uint().unwrap();

    let results = contract.invoke(&function, vec![Value::UInt(BigUint::from(1_u8))], Tag::Latest).await.unwrap();
    let balance_usdt = results[0].as_uint().unwrap();

    let scale = BigUint::from(10000_u32);

    let total = balance_usdc + balance_usdt;
    let scaled_usdc_shares = balance_usdc * &scale / &total;
    let scaled_usdt_shares = balance_usdt * &scale / &total;

    let usdc_shares = scaled_usdc_shares.to_f64().unwrap() / 10000.0;
    let usdt_shares = scaled_usdt_shares.to_f64().unwrap() / 10000.0;

    let share_diff = (usdc_shares - usdt_shares).abs();

    let usdc_amounts = decimalize(&balance_usdc, 6);
    let usdt_amounts = decimalize(&balance_usdt, 6);
    
    println!("USDC: {:>.2}%", usdc_shares * 100.0);
    println!("USDT: {:>.2}%", usdt_shares * 100.0);

    let client = reqwest::Client::new();
    client.post(SLACK_WEBHOOK_URL)
        .json(&serde_json::json!({
            "blocks": [
                {
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": "Ferro Protocol 비율 알림"
                    }
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": ([
                            format!("USDC와 USDT의 비율이 `{:>.2} %p` 만큼 차이가 발생합니다.", share_diff * 100.0),
                        ].join(".\n"))
                    },
                    "fields": [
                        {
                            "type": "mrkdwn",
                            "text": format!("*:usdc: 비율*\n `{:>.2} %`", usdc_shares * 100.0)
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*:usdt: 비율*\n `{:>.2} %`", usdt_shares * 100.0)
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*:usdc: TVL*\n `{}` USDC", usdc_amounts)
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*:usdt: TVL*\n `{}` USDT", usdt_amounts)
                        }
                    ]
                }
            ]
        }))
        .send()
        .await
        .unwrap();

}
