use axum::{
    routing::get,
    extract::{Path, Query, Json},
    Router
};

use std::collections::HashMap;

#[tokio::main]
async fn main() {

    println!("Listening on http://localhost:3000..");

    let app = Router::new().route("/transactions", get(save_transaction));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn save_transaction(Query(params): Query<HashMap<String, String>>) {
    println!("Transaction detected!  Saving..");
    println!("{:?}", params);
}
