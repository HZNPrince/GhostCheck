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

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    // init postgres pool
    let pool = init_db().await;
    create_sessions_table(&pool)
        .await
        .expect("Error creating sessions table");

    // Makes an instance of the AppState to pass to axum
    let state = AppState {
        db: pool,
        client: reqwest::Client::new(),
    };

    let app = routes::create_router(state);
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
