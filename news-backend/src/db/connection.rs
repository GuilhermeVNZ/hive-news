use anyhow::Result;
use sqlx::PgPool;

const DATABASE_URL: &str = "postgresql://postgres:postgres@localhost:5432/news_system";

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let pool = PgPool::connect(DATABASE_URL).await?;
        Ok(Self { pool })
    }
}
