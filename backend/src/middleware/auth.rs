use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde_json::json;

/// Simple token extractor. Token format: "lms_{user_id}"
/// No signature, no expiry — pure MVP.
pub struct UserId(pub String);

impl<S: Send + Sync> FromRequestParts<S> for UserId {
    type Rejection = (StatusCode, axum::Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .unwrap_or("");

        let user_id = auth.strip_prefix("lms_").unwrap_or("").to_string();
        if user_id.is_empty() {
            return Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({
                    "success": false,
                    "message": "Unauthorized",
                    "data": null,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            ));
        }
        Ok(UserId(user_id))
    }
}
