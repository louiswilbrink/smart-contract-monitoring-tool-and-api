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

use ethers_providers::{Provider, Ws, Http, Middleware};

#[tokio::main]
async fn main() {
    let config = load_configuration();

    let provider = Provider::<Http>::try_from(config.provider_url).unwrap();

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
    println!("Transaction detected!  Saving..");
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
        provider_url: var("PROVIDER_URL")?
    })
}

#[derive(Debug)]
pub struct Configuration {
    pub provider_url: String
}
