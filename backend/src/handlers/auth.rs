use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use uuid::Uuid;

use crate::{
    models::{ApiResponse, AuthResponse, LoginPayload, PublicUser, RefreshPayload, RegisterPayload, User, UserRole},
    state::AppState,
};

fn token_for(user_id: &str) -> String {
    format!("lms_{}", user_id)
}

fn refresh_for(user_id: &str) -> String {
    format!("lms_refresh_{}", user_id)
}

fn err(status: StatusCode, msg: &str) -> impl IntoResponse {
    (status, Json(json!({ "success": false, "message": msg, "data": null,
        "timestamp": chrono::Utc::now().to_rfc3339() })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let users = state.users.lock().unwrap();
    let user = users
        .iter()
        .find(|u| u.email.eq_ignore_ascii_case(&payload.email) && u.password == payload.password);

    match user {
        Some(u) => Json(json!(ApiResponse::ok(
            AuthResponse {
                token: token_for(&u.id),
                refresh_token: refresh_for(&u.id),
                expires_in: 86400,
                user: PublicUser::from(u.clone()),
            },
            "Login successful"
        ))).into_response(),
        None => err(StatusCode::UNAUTHORIZED, "Invalid email or password").into_response(),
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let mut users = state.users.lock().unwrap();
    if users.iter().any(|u| u.email.eq_ignore_ascii_case(&payload.email)) {
        return err(StatusCode::CONFLICT, "Email already registered").into_response();
    }
    let id = Uuid::new_v4().to_string();
    let new_user = User {
        id: id.clone(),
        employee_id: payload.employee_id,
        first_name: payload.first_name,
        last_name: payload.last_name,
        email: payload.email,
        password: payload.password,
        role: UserRole::Employee,
        department: payload.department,
        designation: payload.designation,
        manager_id: None, manager_name: None,
        joining_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        avatar_url: None, phone: payload.phone, is_active: true,
        address: None, emergency_contact: None, emergency_phone: None,
    };
    let public = PublicUser::from(new_user.clone());
    users.push(new_user);
    (StatusCode::CREATED, Json(json!(ApiResponse::ok(
        AuthResponse { token: token_for(&id), refresh_token: refresh_for(&id), expires_in: 86400, user: public },
        "Registered successfully"
    )))).into_response()
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshPayload>,
) -> impl IntoResponse {
    // token format: lms_refresh_{user_id}
    let user_id = payload.refresh_token
        .strip_prefix("lms_refresh_")
        .unwrap_or("")
        .to_string();
    if user_id.is_empty() {
        return err(StatusCode::UNAUTHORIZED, "Invalid refresh token").into_response();
    }
    let users = state.users.lock().unwrap();
    match users.iter().find(|u| u.id == user_id) {
        Some(u) => Json(json!(ApiResponse::ok(
            AuthResponse {
                token: token_for(&u.id),
                refresh_token: refresh_for(&u.id),
                expires_in: 86400,
                user: PublicUser::from(u.clone()),
            },
            "Token refreshed"
        ))).into_response(),
        None => err(StatusCode::UNAUTHORIZED, "User not found").into_response(),
    }
}
