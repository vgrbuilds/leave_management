use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    middleware::auth::UserId,
    models::{ApiResponse, ComplianceReport, DashboardStats, PaginatedResponse, PublicUser},
    state::AppState,
};

fn err(status: StatusCode, msg: &str) -> impl IntoResponse {
    (
        status,
        Json(json!({
            "success": false, "message": msg, "data": null,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    )
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserQueryParams {
    pub page: Option<usize>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub year: Option<i32>,
    pub department: Option<String>,
}

// ─── Profile ───────────────────────────────────────────────

pub async fn get_profile(
    State(state): State<AppState>,
    user_id: UserId,
) -> impl IntoResponse {
    let users = state.users.lock().unwrap();
    match users.iter().find(|u| u.id == user_id.0) {
        Some(u) => Json(json!(ApiResponse::ok(PublicUser::from(u.clone()), "OK"))).into_response(),
        None => err(StatusCode::NOT_FOUND, "User not found").into_response(),
    }
}

pub async fn update_profile(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut users = state.users.lock().unwrap();
    match users.iter_mut().find(|u| u.id == user_id.0) {
        Some(u) => {
            if let Some(v) = payload.get("phone").and_then(|v| v.as_str()) { u.phone = Some(v.to_string()); }
            if let Some(v) = payload.get("address").and_then(|v| v.as_str()) { u.address = Some(v.to_string()); }
            if let Some(v) = payload.get("emergencyContact").and_then(|v| v.as_str()) { u.emergency_contact = Some(v.to_string()); }
            if let Some(v) = payload.get("emergencyPhone").and_then(|v| v.as_str()) { u.emergency_phone = Some(v.to_string()); }
            let public = PublicUser::from(u.clone());
            Json(json!(ApiResponse::ok(public, "Profile updated"))).into_response()
        }
        None => err(StatusCode::NOT_FOUND, "User not found").into_response(),
    }
}

pub async fn change_password(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let current = payload.get("currentPassword").and_then(|v| v.as_str()).unwrap_or("");
    let new_pass = payload.get("newPassword").and_then(|v| v.as_str()).unwrap_or("");
    let mut users = state.users.lock().unwrap();
    match users.iter_mut().find(|u| u.id == user_id.0) {
        Some(u) => {
            if u.password != current {
                return err(StatusCode::BAD_REQUEST, "Current password is incorrect").into_response();
            }
            u.password = new_pass.to_string();
            Json(json!(ApiResponse::ok(json!(null), "Password changed"))).into_response()
        }
        None => err(StatusCode::NOT_FOUND, "User not found").into_response(),
    }
}

// ─── Dashboard ─────────────────────────────────────────────

pub async fn get_dashboard_stats(State(state): State<AppState>, _user_id: UserId) -> impl IntoResponse {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let requests = state.leave_requests.lock().unwrap();
    let users = state.users.lock().unwrap();
    let holidays = state.holidays.lock().unwrap();

    let pending = requests.iter().filter(|r| matches!(r.status, crate::models::LeaveStatus::Pending)).count();
    let approved_today = requests.iter().filter(|r| {
        matches!(r.status, crate::models::LeaveStatus::Approved)
            && r.approved_on.as_deref() == Some(&today)
    }).count();
    let on_leave_today = requests.iter().filter(|r| {
        matches!(r.status, crate::models::LeaveStatus::Approved)
            && r.start_date <= today && r.end_date >= today
    }).count();
    let upcoming = holidays.iter().filter(|h| h.date >= today).count();

    Json(json!(ApiResponse::ok(DashboardStats {
        total_employees: users.len(),
        pending_requests: pending,
        approved_today,
        on_leave_today,
        upcoming_holidays: upcoming,
    }, "OK"))).into_response()
}

// ─── HR Admin: all users ───────────────────────────────────

pub async fn get_all_users(
    State(state): State<AppState>,
    _user_id: UserId,
    Query(params): Query<UserQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10);

    let users = state.users.lock().unwrap();
    let filtered: Vec<PublicUser> = users.iter()
        .filter(|u| {
            params.search.as_ref().map_or(true, |s| {
                let q = s.to_lowercase();
                u.first_name.to_lowercase().contains(&q)
                    || u.last_name.to_lowercase().contains(&q)
                    || u.email.to_lowercase().contains(&q)
                    || u.employee_id.to_lowercase().contains(&q)
            })
        })
        .map(|u| PublicUser::from(u.clone()))
        .collect();

    let total = filtered.len();
    let start = ((page - 1) * page_size).min(total);
    let end = (start + page_size).min(total);
    let items = filtered[start..end].to_vec();

    Json(json!(ApiResponse::ok(PaginatedResponse::new(items, total, page, page_size), "OK"))).into_response()
}

pub async fn get_user_by_id(
    State(state): State<AppState>,
    _user_id: UserId,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let users = state.users.lock().unwrap();
    match users.iter().find(|u| u.id == id) {
        Some(u) => Json(json!(ApiResponse::ok(PublicUser::from(u.clone()), "OK"))).into_response(),
        None => err(StatusCode::NOT_FOUND, "User not found").into_response(),
    }
}

pub async fn update_user(
    State(state): State<AppState>,
    _user_id: UserId,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut users = state.users.lock().unwrap();
    match users.iter_mut().find(|u| u.id == id) {
        Some(u) => {
            if let Some(v) = payload.get("isActive").and_then(|v| v.as_bool()) { u.is_active = v; }
            if let Some(v) = payload.get("designation").and_then(|v| v.as_str()) { u.designation = v.to_string(); }
            if let Some(v) = payload.get("department").and_then(|v| v.as_str()) { u.department = v.to_string(); }
            let public = PublicUser::from(u.clone());
            Json(json!(ApiResponse::ok(public, "User updated"))).into_response()
        }
        None => err(StatusCode::NOT_FOUND, "User not found").into_response(),
    }
}

// ─── Compliance Report ────────────────────────────────────

pub async fn get_compliance_report(
    State(state): State<AppState>,
    _user_id: UserId,
    Query(params): Query<UserQueryParams>,
) -> impl IntoResponse {
    let users = state.users.lock().unwrap();
    let requests = state.leave_requests.lock().unwrap();

    let mut dept_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for u in users.iter() {
        dept_map.entry(u.department.clone()).or_default().push(u.id.clone());
    }

    let year_str = params.year.map(|y| y.to_string()).unwrap_or_default();

    let report: Vec<ComplianceReport> = dept_map.iter()
        .filter(|(dept, _)| params.department.as_ref().map_or(true, |d| dept.eq_ignore_ascii_case(d)))
        .map(|(dept, user_ids)| {
            let dept_leaves: Vec<f64> = user_ids.iter().map(|uid| {
                requests.iter()
                    .filter(|r| &r.employee_id == uid
                        && matches!(r.status, crate::models::LeaveStatus::Approved)
                        && (year_str.is_empty() || r.start_date.starts_with(&year_str)))
                    .map(|r| r.total_days)
                    .sum::<f64>()
            }).collect();

            let total_emp = user_ids.len();
            let avg = if total_emp > 0 { dept_leaves.iter().sum::<f64>() / total_emp as f64 } else { 0.0 };
            let utilization = (avg / 21.0 * 100.0).min(100.0);
            let pending = requests.iter()
                .filter(|r| user_ids.contains(&r.employee_id) && matches!(r.status, crate::models::LeaveStatus::Pending))
                .count();

            ComplianceReport {
                department: dept.clone(),
                total_employees: total_emp,
                average_leaves_taken: (avg * 10.0).round() / 10.0,
                leave_utilization_percent: (utilization * 10.0).round() / 10.0,
                excess_leave_count: dept_leaves.iter().filter(|&&d| d > 21.0).count(),
                pending_approvals: pending,
            }
        })
        .collect();

    Json(json!(ApiResponse::ok(report, "OK"))).into_response()
}

// ─── Manager: team members ────────────────────────────────

pub async fn get_team(
    State(state): State<AppState>,
    user_id: UserId,
) -> impl IntoResponse {
    let users = state.users.lock().unwrap();
    let team: Vec<PublicUser> = users.iter()
        .filter(|u| u.manager_id.as_deref() == Some(&user_id.0))
        .map(|u| PublicUser::from(u.clone()))
        .collect();
    Json(json!(ApiResponse::ok(team, "OK"))).into_response()
}
