pub mod sessions;
use std::env;

pub use sessions::*;

use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init_db() -> PgPool {
    let url = env::var("DATABASE_URL").expect("Error fetching DATABASE_URL from env");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect Postgres")
}
