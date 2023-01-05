use axum::{
    routing::get,
    Router
};

#[tokio::main]
async fn main() {

    println!("Listening on http://localhost:3000..");

    let app = Router::new().route("/transactions", get(save_transaction));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn save_transaction() {
    println!("Transaction detected!  Saving..");
}
