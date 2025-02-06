use ccx_mexc::api::um::UmApi;
use ccx_mexc::ApiCred;
use ccx_mexc::BinanceResult;
use ccx_mexc_examples_util::*;

#[actix_rt::main]
async fn main() {
    let _ = main_().await;
}

async fn main_() -> BinanceResult<()> {
    let _ = dotenv::dotenv();
    env_logger::init();

    let binance_usdtm = UmApi::<ApiCred>::from_env();

    println!("Running...");

    print_res(binance_usdtm.ping()?.await)?;

    let time = print_res(binance_usdtm.time()?.await)?.server_time;
    println!("Server Time: {}", time);

    let info = print_res(binance_usdtm.exchange_info()?.await)?;
    for symbol in info.symbols {
        // if &symbol.base_asset != "BTC" {
        //     continue;
        // }
        println!(
            "{:>8}/{:8}\t{:?}\t{:?}\t{:?}",
            symbol.base_asset,
            symbol.quote_asset,
            symbol.contract_type,
            symbol.underlying_type,
            symbol.underlying_sub_type
        );
    }

    Ok(())
}
