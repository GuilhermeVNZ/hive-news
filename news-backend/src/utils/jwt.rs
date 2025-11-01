use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
}

pub struct JwtService;

impl JwtService {
    /// Get JWT secret from environment or use default
    fn get_secret() -> String {
        env::var("JWT_SECRET").unwrap_or_else(|_| {
            "news-system-secret-key-change-in-production".to_string()
        })
    }

    /// Generate JWT token for a user
    pub fn generate_token(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let secret = Self::get_secret();
        let now = Utc::now();
        let exp = (now + Duration::days(7)).timestamp() as usize; // 7 days expiration
        let iat = now.timestamp() as usize;

        let claims = Claims {
            sub: username.to_string(),
            exp,
            iat,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
    }

    /// Verify JWT token and extract claims
    #[allow(dead_code)]
    pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let secret = Self::get_secret();
        let validation = Validation::default();
        
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        )
        .map(|data| data.claims)
    }
}
