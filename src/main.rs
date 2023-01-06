// Next: code outer loop: watch_blocks!  Get timestamp off of it!  Subscribe to events from that
// block!

use dotenv::dotenv;

use std::env::var;
use std::env::VarError;

use std::sync::Arc;

use eyre::Result;

use axum::{
    routing::get,
    extract::{Path, Query, Json},
    Router
};

use std::collections::HashMap;

use ethers::prelude::*;

use ethers::{
    core::{
        abi::AbiDecode,
        types::{Address, BlockNumber, Filter, U256},
    },
};

use ethers_providers::{Provider, Ws};

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_configuration();

    launch_api().await;

    launch_transfer_monitor().await;

    Ok(())
}

async fn launch_transfer_monitor() -> Result<()> {
    // Set up client/provider; using open Infura endpoint on mainnet.
    let client = Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27").await?;

    let client = Arc::new(client);

    // Start monitoring on the last block.
    // TODO: check database for the last block that transfer events were saved.
    let last_block = client.get_block(BlockNumber::Latest).await?.unwrap().number.unwrap();

    // Filter logs by tranfer events on the specified block.
    let erc20_transfer_filter = Filter::new().from_block(last_block).event("Transfer(address,address,uint256)");

    // Subscribe to the logs using the filter, saved to a stream.
    let mut stream = client.subscribe_logs(&erc20_transfer_filter).await?.take(2);

    while let Some(log) = stream.next().await {
        println!(
            "block: {:?}, tx: {:?}, token: {:?}, from: {:?}, to: {:?}, amount: {:?}",
            log.block_number,
            log.transaction_hash,
            log.address,
            Address::from(log.topics[1]),
            Address::from(log.topics[2]),
            U256::decode(log.data),
        );
    }

    Ok(())
}

async fn launch_api() -> Result<()> {
    println!("Listening on http://localhost:3000..");

    let app = Router::new().route("/transactions", get(process_transaction));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn process_transaction(Query(params): Query<HashMap<String, String>>) {
    println!("Querying transactions..");
    println!("{:?}", params);
}

pub fn load_configuration() -> Configuration {
    let config = read_environment_variables();

    match config {
        Ok(config) => config,
        Err(_) => panic!("Missing environment variables in the `.env` file.")
    }
}

fn read_environment_variables() -> Result<Configuration, VarError> {
    dotenv().ok();

    Ok(Configuration {
        contract_address: var("CONTRACT_ADDRESS")?
    })
}

#[derive(Debug)]
pub struct Configuration {
    pub contract_address: String
}
