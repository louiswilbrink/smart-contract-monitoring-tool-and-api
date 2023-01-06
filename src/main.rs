use dotenv::dotenv;

use std::env::var;
use std::env::VarError;

use axum::{
    routing::get,
    extract::{Path, Query, Json},
    Router
};

use std::collections::HashMap;

use ethers::prelude::*;

use ethers_providers::MAINNET;

use ethers_providers::{Provider, Ws, Http, Middleware};

#[tokio::main]
async fn main() {
    let config = load_configuration();

    println!("Configuration: {:?}", config);

    let provider = MAINNET.provider();

    match provider.client_version().await {
        Ok(version) => println!("The version is {}", version),
        Err(error) => println!("Error getting client version! {:?}", error)
    }

    match provider.node_client().await.expect("Error getting node client") {
        Geth => println!("Provider client is running Geth."),
        _ => println!("Provider client is not running Geth.")
    }

    //provider.subscribe("newPendingTransactions");

    //provider.unsubscribe("newPendingTransactions");

    println!("Listening on http://localhost:3000..");

    let app = Router::new().route("/transactions", get(process_transaction));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
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
