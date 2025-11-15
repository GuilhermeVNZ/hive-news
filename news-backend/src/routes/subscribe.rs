use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::utils::path_resolver;

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    email: String,
}

#[derive(Debug, Serialize)]
pub struct SubscribeResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    already_subscribed: Option<bool>,
}

/// Subscribe endpoint for ScienceAI
pub async fn subscribe(Json(payload): Json<SubscribeRequest>) -> impl IntoResponse {
    // Validate email
    let email = payload.email.trim().to_lowercase();
    
    if email.is_empty() || !email.contains('@') {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid email address"
            })),
        ).into_response();
    }

    // Get subscribers file path
    let workspace_root = path_resolver::workspace_root();
    let subscribers_path = workspace_root.join("scienceai_subscribers.json");

    // Read existing subscribers
    let mut subscribers: serde_json::Value = if subscribers_path.exists() {
        match std::fs::read_to_string(&subscribers_path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(data) => data,
                    Err(_) => json!({
                        "emails": [],
                        "subscribedAt": {}
                    }),
                }
            }
            Err(_) => json!({
                "emails": [],
                "subscribedAt": {}
            }),
        }
    } else {
        json!({
            "emails": [],
            "subscribedAt": {}
        })
    };

    // Check if already subscribed
    let emails = subscribers["emails"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    
    if emails.iter().any(|e| e.as_str() == Some(&email)) {
        return (
            StatusCode::OK,
            Json(SubscribeResponse {
                success: true,
                message: "Email already subscribed".to_string(),
                already_subscribed: Some(true),
            }),
        ).into_response();
    }

    // Add email
    if let Some(emails_array) = subscribers["emails"].as_array_mut() {
        emails_array.push(json!(email));
        // Sort by converting to strings, sorting, then converting back
        let mut email_strings: Vec<String> = emails_array
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        email_strings.sort();
        *emails_array = email_strings.into_iter().map(|s| json!(s)).collect();
    }

    // Add subscription timestamp
    if let Some(subscribed_at) = subscribers["subscribedAt"].as_object_mut() {
        subscribed_at.insert(
            email.clone(),
            json!(chrono::Utc::now().to_rfc3339()),
        );
    }

    // Save to file
    match std::fs::write(
        &subscribers_path,
        serde_json::to_string_pretty(&subscribers).unwrap_or_default(),
    ) {
        Ok(_) => (
            StatusCode::OK,
            Json(SubscribeResponse {
                success: true,
                message: "Email subscribed successfully".to_string(),
                already_subscribed: Some(false),
            }),
        ).into_response(),
        Err(e) => {
            eprintln!("Failed to save subscribers file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to subscribe email"
                })),
            ).into_response()
        }
    }
}

