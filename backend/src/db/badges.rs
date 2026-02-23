use sqlx::PgPool;

// Creates minted_repos table if not exists
pub async fn create_badge_table(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS minted_repos (
            id SERIAL PRIMARY KEY,
            wallet_address TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            minted_at TIMESTAMP DEFAULT NOW(),
            UNIQUE(wallet_address, repo_name)
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Save a minted_repo badge record
pub async fn insert_minted_repo(
    pool: &PgPool,
    wallet_addr: &str,
    repo_name: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
            INSERT INTO minted_repos (wallet_address, repo_name)
            Values ($1, $2)
            ON CONFLICT (wallet_address, repo_name) DO NOTHING
        "#,
    )
    .bind(wallet_addr)
    .bind(repo_name)
    .execute(pool)
    .await?;

    Ok(())
}

// Get all the minted_repos for a wallet
pub async fn fetch_all_for_wallet(pool: &PgPool, wallet_addr: &str) -> anyhow::Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT repo_name FROM minted_repos
        WHERE wallet_address = $1
        ORDER BY minted_at DESC
       "#,
    )
    .bind(wallet_addr)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}
