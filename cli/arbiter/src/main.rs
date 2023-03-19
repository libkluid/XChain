extern crate arbiter;
use contracts::eth::EthereumFunction;
use itertools::Itertools;
use ethabi::Value;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    log::info!("Initializing...");
    let wemix = arbiter::defi::wemix::WemixNetwork::websocket("wss://ws.wemix.com");

    let multicall_address = "0xDC88a401068EC15E6F44a038052F382D2Bf56548";
    let tokens = vec![
        wemix.token("0x7D72b22a74A216Af4a002a1095C8C707d6eC1C5f", Some(multicall_address)).await, // wrapped_wemix
        wemix.token("0x8E81fCc2d4A3bAa0eE9044E0D7E36F59C9BbA9c1", Some(multicall_address)).await, // wemix_dollar
        wemix.token("0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D", Some(multicall_address)).await, // usdc
        wemix.token("0x461d52769884ca6235B685EF2040F47d30C94EB5", Some(multicall_address)).await, // klay
        wemix.token("0xA649325Aa7C5093d12D6F98EB4378deAe68CE23F", Some(multicall_address)).await, // usdt
        wemix.token("0x9B377bd7Db130E8bD2f3641E0E161cB613DA93De", Some(multicall_address)).await, // staked_wemix
        wemix.token("0x765277EebeCA2e31912C9946eAe1021199B39C61", Some(multicall_address)).await, // ethereum
        wemix.token("0x2C78f1b70Ccf63CDEe49F9233e9fAa99D43AA07e", Some(multicall_address)).await, // wbtc
        wemix.token("0x2B58644b9f210ebB8fBF4C27066f9d1d97B03CBc", Some(multicall_address)).await, // wrft
        wemix.token("0xC1Be9a4D5D45BeeACAE296a7BD5fADBfc14602C4", Some(multicall_address)).await, // wbnb
        wemix.token("0xe6801928061CDbE32AC5AD0634427E140EFd05F9", Some(multicall_address)).await, // kleva
        wemix.token("0x70f1F317697337d297F5338d3dD72a6C4C51BDE1", Some(multicall_address)).await, // tipo
    ];

    log::info!("Registered {} tokens", tokens.len());
    for (index, token) in tokens.iter().enumerate() {
        log::info!("{:>2}. {} [dec: {:>2}, sym: {}]", 1+index, token.address(), token.decimals(), token.symbol());
    }

    log::info!("WemixFi contracts initializing...");
    let wemixfi = wemix.wemixfi("0x80a5A916FB355A8758f0a3e47891dc288DAC2665").await;
    log::info!("WemixFi contracts initializing... done");

    log::info!("Fetching all combinations...");
    let mut pairs = vec![];
    for pair in tokens.iter().combinations(2) {
        let pair = wemixfi.factory.pair(pair[0].clone(), pair[1].clone()).await;

        if pair.address() != "0x0000000000000000000000000000000000000000" {
            pairs.push(pair);
        }
    }
    log::info!("Found {} pairs", pairs.len());

    for (index, pair) in pairs.iter().enumerate() {
        let token0 = pair.token0();
        let token1 = pair.token1();
        let pair_name = format!("{}-{}", token0.symbol(), token1.symbol());
        log::info!("{:>2}. {} [name: {}]", 1+index, pair.address(), pair_name);
    }

    let multicall = wemix.contract(multicall_address);
    let aggregate = EthereumFunction::new("aggregate", &["(address,bytes)[]"], &["uint256", "bytes[]"]).unwrap();

    let get_reserves = EthereumFunction::new("getReserves", &[], &["uint112", "uint112", "uint32"]).unwrap();
    let values = pairs.iter().map(|pair| {
        let pair_address = Value::address(pair.address()).unwrap();
        let data = Value::Bytes(get_reserves.encode(vec![]).unwrap());
        Value::Tuple(vec![pair_address, data])
    }).collect::<Vec<_>>();
    let args = vec![Value::Array(values)];

    loop {
        let instant = std::time::Instant::now();
        let results = multicall.invoke(&aggregate, args.clone()).await.unwrap();
        let block_number = results[0].as_uint().unwrap();
        log::info!("Block: {}", block_number);

        let reserves = results[1].as_array().unwrap();
        for (index, pair) in pairs.iter().enumerate() {
            let token0 = pair.token0();
            let token1 = pair.token1();
            let pair_name = format!("{}-{}", token0.symbol(), token1.symbol());

            let bytes = reserves[index].as_bytes().unwrap();
            let reserves = get_reserves.decode(bytes).unwrap();
            let reserve0 = reserves[0].as_uint().unwrap();
            let reserve1 = reserves[1].as_uint().unwrap();
            log::info!("{:<16} {} = {} {} = {}", pair_name, token0.symbol(), reserve0, token1.symbol(), reserve1);
        }
        let elapsed = instant.elapsed();
        log::debug!("RPS: {:.2}", 1_000_000_f64 / elapsed.as_micros() as f64);
    }
}
