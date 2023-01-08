use std::{
    env::{var, VarError},
    time::Duration,
    collections::HashMap,
};

use axum::{
    routing::get,
    extract::{Path, Query, Json, State},
    http::{StatusCode},
    Router,
};

use ethers::{
    prelude::*,
    core::{
        abi::AbiDecode,
        types::{Address, Filter, U256},
    },
};

use ethers_providers::{Provider, Ws};

use sqlx::{
    Pool, Row,
    postgres::{Postgres, PgPool, PgPoolOptions, PgRow},
};

use serde::Serialize;

use dotenv::dotenv;

use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_configuration();
    
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|i| i == "--api") {
        launch_api(&config).await.expect("always on");
    } else {
        launch_transfer_monitor(&config).await.expect("always on");
    }

    Ok(())
}

async fn launch_transfer_monitor(config: &Configuration) -> Result<()> {
    let pool = get_connection_pool(&config).await.unwrap();

    // TODO: corral into get_provider().
    let ws_endpoint = "wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27";
    let ws = Ws::connect(ws_endpoint).await?;
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));

    println!("Monitoring blocks..");

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
            let _row: (i64,) = sqlx::query_as(
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
        .route(
            "/transactions/:tx_hash",
            get(get_transaction_by_hash)
        )
        .with_state(pool);

    println!("Listening on http://localhost:3000..");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn get_transaction_by_hash(State(pool): State<PgPool>,  Path(tx_hash): Path<String>) -> Result<Json<Transfer>, (StatusCode, String)> {

    // TODO: String manipulation not ideal or easy to read; use ORM.
    let mut query = String::from("SELECT * FROM transfers WHERE");

    query.push_str(" tx_hash='");
    query.push_str(&tx_hash);
    query.push_str("'");

    println!("Query: {}", query);

    // TODO: convert to query_as using custom struct.
    let row = sqlx::query(&query)
        .fetch_one(&pool)
        .await;

    let transfer = match row {
        Ok(r) => Transfer { 
            tx_hash: r.get("tx_hash"),
            sender: r.get("sender"),
            recipient: r.get("recipient"),
            amount: r.get("amount"),
            timestamp: r.get("timestamp"),
        },
        // TODO: Add proper error response in the future.
        Err(..) => Transfer {
            tx_hash: String::from("0xNotFound"),
            sender: String::from("0xNotFound"),
            recipient: String::from("0xNotFound"),
            amount: 0.0,
            timestamp: 0
        },
    };

    Ok(Json(transfer))
}

async fn get_transactions(State(pool): State<PgPool>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Vec<Transfer>>, (StatusCode, String)> {

    // TODO: Find a better way to build filtering.  Using string manipulation for now since sqlx is
    // not an ORM.
    let mut query = String::from("SELECT * FROM transfers");

    query.push_str(" WHERE 1=1 "); // WHERE clause added so we can add AND clauses conditionally.

    if params.contains_key("sender") {
        query.push_str("AND sender='");
        query.push_str(&params.get("sender").unwrap());
        query.push_str("'");
    }

    if params.contains_key("recipient") {
        query.push_str(" AND recipient='");
        query.push_str(&params.get("recipient").unwrap());
        query.push_str("'");
    }

    if params.contains_key("minAmount") {
        query.push_str(" AND amount > ");
        query.push_str(&params.get("minAmount").unwrap());
    }

    if params.contains_key("maxAmount") {
        query.push_str(" AND amount < ");
        query.push_str(&params.get("maxAmount").unwrap());
    }

    if params.contains_key("before") {
        query.push_str(" AND timestamp < ");
        query.push_str(&params.get("before").unwrap());
    }

    if params.contains_key("after") {
        query.push_str(" AND timestamp > ");
        query.push_str(&params.get("after").unwrap());
    }

    if params.contains_key("order") {
        if params.get("order").unwrap().eq("asc") {
            query.push_str(" ORDER BY id");
        }

        if params.get("order").unwrap().eq("desc") {
            query.push_str(" ORDER BY id DESC");
        }
    }

    if params.contains_key("limit") {
        query.push_str(" LIMIT ");
        query.push_str(&params.get("limit").unwrap());
    }

    if params.contains_key("offset") {
        query.push_str(" OFFSET ");
        query.push_str(&params.get("offset").unwrap());
        query.push_str(" ROWS");
    }

    println!("Query: {}", query);

    // TODO: convert to query_as using custom struct.
    let rows: Vec<PgRow> = sqlx::query(&query)
        .fetch_all(&pool)
        .await
        .unwrap();

    let transfers: Vec<Transfer> = rows
        .iter()
        .map(|r| Transfer { 
            tx_hash: r.get("tx_hash"),
            sender: r.get("sender"),
            recipient: r.get("recipient"),
            amount: r.get("amount"),
            timestamp: r.get("timestamp"),
        })
        .collect::<Vec<Transfer>>();

    Ok(Json(transfers))
}

async fn get_connection_pool(config: &Configuration) -> Result<Pool<Postgres>, sqlx::Error> {
    println!("Connecting to database..\n");

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
        
    // TODO: Disallow multiple seeding.
    let _row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO transfers (tx_hash, sender, recipient, amount, timestamp)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
        )
        .bind("0x87327201eac4cf8df4ec4831e434ffd872b7b4eb")
        .bind("0x00000000a991c429ee2ec6df19d40fe0c80088b8")
        .bind("0xc5be99a02c6857f9eac67bbce58df5572498f40c")
        .bind(34005500222.0)
        .bind(1673102315)
        .fetch_one(&pool)
        .await?;

    let _row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO transfers (tx_hash, sender, recipient, amount, timestamp)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
        )
        .bind("0xb653d23e95a02cb6de0bef363406ee617a545cd4")
        .bind("0x62716cd006b3c64ca1ef5dc439b56069e16cbe9c")
        .bind("0x5c6eff81d09cb1ebe7ce87f4f2df638f83b05b6c")
        .bind(464332344332.0)
        .bind(1673101315)
        .fetch_one(&pool)
        .await?;

    let _row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO transfers (tx_hash, sender, recipient, amount, timestamp)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
        )
        .bind("0x1f7259c7940891146dd6888d7f707b4a889844d7")
        .bind("0x8c0f2dbabe1fe6d79bbc90930de313da9c81c8bd")
        .bind("0xf267d62188820f1ce7aebf01bfdefc5d4c45d3cf")
        .bind(77238333.0)
        .bind(1673101200)
        .fetch_one(&pool)
        .await?;

    Ok(pool)
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

fn _print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

#[derive(Debug)]
pub struct Configuration {
    pub database_name: String,
    pub database_username: String,
    pub database_password: String,
}

#[derive(Serialize, Debug)]
struct Transfer {
    tx_hash: String,
    sender: String, // TODO: Convert to Ethereum Address type.
                    // For now, since it's only being sent as JSON to the client,
                    // we can use String for simplicity.
    recipient: String,
    amount: f64,
    timestamp: i64,
}

