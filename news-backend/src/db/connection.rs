use std::env;

use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};

const DEFAULT_DATABASE_URL: &str = "postgresql://postgres:postgres@localhost:5432/news_system";

#[allow(dead_code)]
#[derive(Clone)]
pub struct Database {
    pub pool: Option<PgPool>,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            eprintln!(
                "ℹ️  DATABASE_URL not set, falling back to default ({})",
                DEFAULT_DATABASE_URL
            );
            DEFAULT_DATABASE_URL.to_string()
        });

        // Try to connect to database, but allow server to start without it
        // Auth and config endpoints use file-based storage, so they work without DB
        let pool = match PgPoolOptions::new().connect(&database_url).await {
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
