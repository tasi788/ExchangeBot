use grammers_client::{Client, Config};
use grammers_session::Session;
use grammers_tl_types as tl;
use tokio::runtime;

use serde::Serialize;

mod lib;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

async fn async_main() -> Result {
    let config: lib::Config = lib::utils::load_config().unwrap();
    println!("Connecting to Telegram...");
    let client = Client::connect(Config {
        session: Session::load_file_or_create("ping.session")?,
        api_id: config.api_id,
        api_hash: config.api_hash,
        params: Default::default(),
    })
        .await?;
    println!("Connected!");

    println!("Sending ping...");
    dbg!(client.invoke(&tl::functions::Ping { ping_id: 0 }).await?);
    println!("Ping sent successfully!");

    Ok(())
}

fn main() -> Result {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}
