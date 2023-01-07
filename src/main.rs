use dotenv::dotenv;

use std::env::var;
use std::env::VarError;

use std::time::Duration;

use std::sync::Arc;

use eyre::Result;

use axum::{
    routing::get,
    extract::{Path, Query, Json, State},
    http::{request::Parts, StatusCode},
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

use sqlx::{Pool,Row};
use sqlx::postgres::{Postgres, PgPool, PgPoolOptions, PgRow};

use primitive_types::H256;

use conv::*;

use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_configuration();

    launch_api(&config).await;

    //launchTransferMonitor(&config).await;

    Ok(())
}

async fn launchTransferMonitor(config: &Configuration) -> Result<()> {
    println!("Monitoring blocks..");

    let pool = get_connection_pool(&config).await.unwrap();

    // TODO: corral into get_provider().
    let ws_endpoint = "wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27";
    let ws = Ws::connect(ws_endpoint).await?;
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));

    let mut stream = provider.watch_blocks().await?.take(20);

    while let Some(block) = stream.next().await {
        let block = provider.get_block(block).await?.unwrap();

        let block_timestamp = block.timestamp;

        let block_timestamp = block_timestamp.as_u64();

        let block_timestamp = block_timestamp as i64;

        println!("");
        println!(
            "Ts: {:?}, block number: {} -> {:?}",
            block.timestamp,
            block.number.unwrap(),
            block.hash.unwrap()
        );

        // Filter logs by tranfer events on the specified block.
        let erc20_transfer_filter = Filter::new().from_block(block.number.unwrap()).event("Transfer(address,address,uint256)");

        // Subscribe to the logs using the filter and send them to a stream for processing.
        // Temporary: Limit to 2 for easy debugging.
        let mut stream = provider.subscribe_logs(&erc20_transfer_filter).await?.take(2);

        while let Some(log) = stream.next().await {
            // Convert log values.
            let tx_hash = format!("{:?}", Address::from(log.transaction_hash.unwrap()));

            let sender = format!("{:?}", Address::from(log.topics[1]));

            let recipient = format!("{:?}", Address::from(log.topics[2]));

            // Convert to f64 for SQL datatype compatibility.
            let amount = U256::decode(log.data).unwrap();

            let amount = amount.as_u128();

            let amount = amount as f64;

            // Display.
            println!("");
            println!(
                "block: {:?}, tx: {:?}, token: {:?}, from: {:?}, to: {:?}, amount: {:?}, timestamp: {:?}",
                log.block_number,
                tx_hash,
                log.address,
                sender,
                recipient,
                amount,
                block_timestamp,
            );

            // Save to database.  TODO: requires error handling.
            let row: (i64,) = sqlx::query_as(
                r#"
                INSERT INTO transfers (tx_hash, sender, recipient, amount, timestamp)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id
                "#
                )
                .bind(tx_hash)
                .bind(sender)
                .bind(recipient)
                .bind(amount)
                .bind(block_timestamp)
                .fetch_one(&pool)
                .await?;
        }
    }

    Ok(())
}

async fn launch_api(config: &Configuration) -> Result<()> {
    let pool = get_connection_pool(&config).await.unwrap();

    let app = Router::new()
        .route(
            "/transactions", 
            get(get_transactions)
        )
        .with_state(pool);

    println!("Listening on http://localhost:3000..");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[derive(Serialize, Debug)]
struct Transfer {
    sender: String, // TODO: Convert to Ethereum Address type.
                   // For now, since it's only being sent as JSON to the client,
                   // we can keep it String for simplicity.
}

async fn get_transactions(State(pool): State<PgPool>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Vec<Transfer>>, (StatusCode, String)> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"
        SELECT sender
        FROM transfers
        "#
        )
        .fetch_all(&pool)
        .await
        .unwrap();

    let transfers: Vec<Transfer> = rows
        .iter()
        .map(|r| Transfer { sender: r.get::<String, _>("sender") })
        .collect::<Vec<Transfer>>();

    Ok(Json(transfers))
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
        database_name: var("DATABASE_NAME")?,
        database_username: var("DATABASE_USERNAME")?,
        database_password: var("DATABASE_PASSWORD")?,
    })
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

async fn get_connection_pool(config: &Configuration) -> Result<Pool<Postgres>, sqlx::Error> {
    println!("Connecting to database..");

    let connection_string = format!("postgres://{:1}:{:2}@localhost/{:3}", config.database_username, config.database_password, config.database_name);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await?;

    // TODO: Add "Successfully connected to database" notification.

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transfers (
            id bigserial,
            tx_hash text,
            sender text,
            recipient text,
            amount float8,
            timestamp bigint
        );"#,
        )
        .execute(&pool)
        .await?;

    Ok(pool)
}

#[derive(Debug)]
pub struct Configuration {
    pub database_name: String,
    pub database_username: String,
    pub database_password: String,
}
