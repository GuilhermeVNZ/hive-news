use axum::{extract::Extension, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::utils::jwt::JwtService;
use crate::db::connection::Database;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub message: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub role: String,
}

/// Simple file-based user storage for now
/// In production, use database
struct UserStore {
    users_path: std::path::PathBuf,
}

impl UserStore {
    fn new() -> Self {
        let path = Path::new("users.json");
        Self {
            users_path: path.to_path_buf(),
        }
    }

    fn load_users(&self) -> HashMap<String, UserData> {
        if !self.users_path.exists() {
            // Create default admin user
            let default_user = UserData {
                username: "admin".to_string(),
                password_hash: hash("123admin123", DEFAULT_COST).unwrap_or_default(),
                role: "admin".to_string(),
            };
            
            let mut users = HashMap::new();
            users.insert("admin".to_string(), default_user.clone());
            
            // Save default user
            self.save_users(&users);
            return users;
        }

        let content = fs::read_to_string(&self.users_path).unwrap_or_default();
        let mut users: HashMap<String, UserData> = serde_json::from_str(&content).unwrap_or_default();
        
        // Update admin password if it exists (password change)
        if let Some(admin_user) = users.get_mut("admin") {
            // Always update to the new password
            admin_user.password_hash = hash("123admin123", DEFAULT_COST).unwrap_or_default();
            self.save_users(&users);
        }
        
        users
    }

    fn save_users(&self, users: &HashMap<String, UserData>) {
        let content = serde_json::to_string_pretty(users).unwrap_or_default();
        fs::write(&self.users_path, content).ok();
    }

    fn verify_user(&self, username: &str, password: &str) -> bool {
        let users = self.load_users();
        
        if let Some(user) = users.get(username) {
            verify(password, &user.password_hash).unwrap_or(false)
        } else {
            false
        }
    }

    fn create_user(&self, username: String, password: String, role: String) -> Result<(), String> {
        let mut users = self.load_users();
        
        if users.contains_key(&username) {
            return Err("User already exists".to_string());
        }

        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|_| "Failed to hash password")?;

        users.insert(username.clone(), UserData {
            username: username.clone(),
            password_hash,
            role,
        });

        self.save_users(&users);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserData {
    username: String,
    password_hash: String,
    role: String,
}

/// Login endpoint
pub async fn login(
    Extension(_db): Extension<Arc<Database>>,
    Json(request): Json<LoginRequest>,
) -> Json<LoginResponse> {
    let user_store = UserStore::new();

    if user_store.verify_user(&request.username, &request.password) {
        match JwtService::generate_token(&request.username) {
            Ok(token) => Json(LoginResponse {
                success: true,
                token: Some(token),
                message: "Login successful".to_string(),
            }),
            Err(e) => Json(LoginResponse {
                success: false,
                token: None,
                message: format!("Failed to generate token: {}", e),
            }),
        }
    } else {
        Json(LoginResponse {
            success: false,
            token: None,
            message: "Invalid username or password".to_string(),
        })
    }
}

/// Logout endpoint (client-side token removal)
pub async fn logout(
    Extension(_db): Extension<Arc<Database>>,
) -> Json<Value> {
    // Logout is primarily handled client-side by removing the token
    // This endpoint just confirms the logout
    Json(json!({
        "success": true,
        "message": "Logged out successfully",
    }))
}

/// Get current user info
pub async fn get_me(
    Extension(_db): Extension<Arc<Database>>,
) -> Json<Value> {
    // In a real implementation, extract user from JWT token in middleware
    // For now, return a default response
    Json(json!({
        "success": true,
        "user": {
            "username": "admin",
            "role": "admin",
        }
    }))
}

/// Change password
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    Extension(_db): Extension<Arc<Database>>,
    Json(request): Json<ChangePasswordRequest>,
) -> Json<Value> {
    // In real implementation, get username from JWT token
    let username = "admin"; // TODO: Extract from token
    
    let user_store = UserStore::new();

    if !user_store.verify_user(username, &request.current_password) {
        return Json(json!({
            "success": false,
            "error": "Current password is incorrect",
        }));
    }

    match user_store.create_user(
        username.to_string(),
        request.new_password,
        "admin".to_string(),
    ) {
        Ok(_) => Json(json!({
            "success": true,
            "message": "Password changed successfully",
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e,
        })),
    }
}
