use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};

const DATABASE_URL: &str = "postgresql://postgres:postgres@localhost:5432/news_system";

#[allow(dead_code)]
#[derive(Clone)]
pub struct Database {
    pub pool: Option<PgPool>,
}

impl Database {
    pub async fn new() -> Result<Self> {
        // Try to connect to database, but allow server to start without it
        // Auth and config endpoints use file-based storage, so they work without DB
        let pool = match PgPoolOptions::new().connect(DATABASE_URL).await {
            Ok(p) => {
                eprintln!("✅ Connected to database");
                Some(p)
            }
            Err(e) => {
                eprintln!("⚠️  Database not available: {} (continuing without database)", e);
                eprintln!("   Auth and config endpoints will work (using file-based storage)");
                eprintln!("   Some endpoints (pages/sources) may not work without database");
                None
            }
        };
        Ok(Self { pool })
    }
}
