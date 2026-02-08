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

#[tokio::main]
async fn main() {
    let app = routes::create_router();
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
