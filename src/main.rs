mod blockchain;

use axum::{
    routing::{get, post},
    Router,
    extract::State,
    Json,
};
use std::sync::{Arc, Mutex};
use blockchain::Blockchain;
use serde::Deserialize;
use tower_http::services::ServeDir;
use std::collections::HashMap;

type AppState = Arc<Mutex<Blockchain>>;

#[tokio::main]
async fn main() {

    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    let app = Router::new()
        .route("/chain", get(get_chain))
        .route("/mine", post(mine_block))
        .route("/validate", get(validate_chain))
        .route("/transfer", post(transfer))
        .route("/balances", get(get_balances))
        //.route("/fork_test", get(test_fork))
        .route("/mempool", get(get_mempool))
        .nest_service("/", ServeDir::new("static"))
        .with_state(blockchain);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running at http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn get_chain(State(state): State<AppState>) -> Json<Vec<blockchain::Block<Vec<blockchain::Transaction>>>> {

    let bc = state.lock().unwrap();

    Json(bc.chain.clone())
}

async fn mine_block(State(state): State<AppState>) -> String {

    let mut bc = state.lock().unwrap();

    bc.mine_pending_transaction();

    "Block mined".to_string()
}

async fn validate_chain(State(state): State<AppState>) -> String {

    let bc = state.lock().unwrap();

    format!("Chain valid: {}", bc.is_chain_valid())
}

#[derive(Deserialize)]
struct TransferRequest {
    sender: String,
    receiver: String,
    amount: u64,
}

async fn transfer(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> String {

    println!("Transfer request received!");

    let mut bc = state.lock().unwrap();

    match bc.transfer(&req.sender, &req.receiver, req.amount) {
        Ok(_) => "Transaction added".to_string(),
        Err(e) => e,
    }
}

async fn get_balances(State(state): State<AppState>) -> Json<HashMap<String,u64>> {

    let bc = state.lock().unwrap();

    Json(bc.balances.clone())
}

async fn get_mempool(State(state): State<AppState>) -> Json<Vec<blockchain::Transaction>> {

    let bc = state.lock().unwrap();
    let pool = bc.mempool.lock().unwrap();
    Json(pool.clone())
}


