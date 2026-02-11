use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub session_id: String,
    pub access_token: String,
    pub username: String,
}

pub async fn create_sessions_table(pool: &PgPool) -> anyhow::Result<()> {
    // Creates a session table in the ghostcheck database
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            session_id TEXT PRIMARY KEY,
            access_token TEXT NOT NULL,
            username TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Inserts a session field consisting of (session_id, access_token, username) in the sessions table
pub async fn insert_session(
    pool: &PgPool,
    access_token: &str,
    username: &str,
) -> anyhow::Result<String> {
    let session_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO sessions (session_id, access_token, username)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(&session_id)
    .bind(access_token)
    .bind(username)
    .execute(pool)
    .await?;

    Ok(session_id)
}

pub async fn get_session(pool: &PgPool, session_id: &str) -> anyhow::Result<Session> {
    let session = sqlx::query_as::<_, Session>(
        r#"
            SELECT session_id, access_token, username
            FROM sessions
            WHERE session_id = $1
        "#,
    )
    .bind(session_id)
    .fetch_one(pool)
    .await?;

    Ok(session)
}
