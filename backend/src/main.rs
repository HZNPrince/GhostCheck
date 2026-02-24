use std::{env, sync::Arc};

use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use tokio::net::TcpListener;

pub mod gh_auth;
pub use gh_auth::*;

pub mod routes;
pub use routes::*;

pub mod gh_api;
pub use gh_api::*;

pub mod models;
pub use models::*;

pub mod signer;
pub use signer::*;

pub mod db;
pub use db::*;

pub mod onchain;

use crate::badges::create_badge_table;

#[tokio::main]
async fn main() {
    // init postgres pool
    let pool = init_db().await;

    // initialize the sessions table
    create_sessions_table(&pool)
        .await
        .expect("Error creating sessions table");

    // initialize the minted_repos table
    create_badge_table(&pool)
        .await
        .expect("Error created mint_repos table");

    // Makes an instance of the AppState to pass to axum
    let rpc_url = env::var("SOLANA_RPC_URL")
        .map_err(|_| "Error fetching SOLANA_RPC_URL from env".to_string())
        .unwrap();
    let state = AppState {
        db: pool,
        client: reqwest::Client::new(),
        rpc: Arc::new(RpcClient::new(rpc_url)),
    };

    let app = routes::create_router(state);
    // Railway sets PORT env var; fallback to 3000 for local dev
    let port = env::var("PORT").unwrap_or("3000".into());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server running on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
