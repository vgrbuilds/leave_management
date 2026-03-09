use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    middleware::auth::UserId,
    models::{
        ApiResponse, Holiday, LeaveActionPayload, LeaveBalance, LeaveDuration, LeaveRequest,
        LeaveStatus, LeaveType, PaginatedResponse,
    },
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

fn today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

// ─── Pagination helper ─────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PageParams {
    pub page: Option<usize>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<usize>,
    pub status: Option<String>,
    #[serde(rename = "leaveType")]
    pub leave_type: Option<String>,
    pub department: Option<String>,
    pub month: Option<u32>,
    pub year: Option<i32>,
}

fn paginate<T: Clone>(items: Vec<T>, page: usize, page_size: usize) -> (Vec<T>, usize) {
    let total = items.len();
    let start = ((page - 1) * page_size).min(total);
    let end = (start + page_size).min(total);
    (items[start..end].to_vec(), total)
}

// ─── Employee: apply leave ─────────────────────────────────

pub async fn apply_leave(
    State(state): State<AppState>,
    user_id: UserId,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let users = state.users.lock().unwrap().clone();
    let user = match users.iter().find(|u| u.id == user_id.0) {
        Some(u) => u.clone(),
        None => return err(StatusCode::UNAUTHORIZED, "User not found").into_response(),
    };

    let mut leave_type_str = String::new();
    let mut start_date = String::new();
    let mut end_date = String::new();
    let mut duration_str = String::new();
    let mut reason = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "leaveType" => leave_type_str = field.text().await.unwrap_or_default(),
            "startDate" => start_date = field.text().await.unwrap_or_default(),
            "endDate" => end_date = field.text().await.unwrap_or_default(),
            "duration" => duration_str = field.text().await.unwrap_or_default(),
            "reason" => reason = field.text().await.unwrap_or_default(),
            _ => { let _ = field.bytes().await; }
        }
    }

    let leave_type = parse_leave_type(&leave_type_str);
    let duration = parse_duration(&duration_str);

    let total_days: f64 = match duration {
        LeaveDuration::FullDay => {
            // rough calc: count weekdays between start and end
            days_between(&start_date, &end_date)
        }
        _ => 0.5,
    };

    let new_req = LeaveRequest {
        id: format!("LR{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("000").to_uppercase()),
        employee_id: user.id.clone(),
        employee_name: format!("{} {}", user.first_name, user.last_name),
        department: user.department.clone(),
        leave_type,
        start_date,
        end_date,
        duration,
        total_days,
        reason,
        status: LeaveStatus::Pending,
        applied_on: today(),
        approved_by: None,
        approver_name: None,
        approved_on: None,
        rejection_reason: None,
        attachment_url: None,
    };

    state.leave_requests.lock().unwrap().push(new_req.clone());
    Json(json!(ApiResponse::ok(new_req, "Leave applied successfully"))).into_response()
}

fn days_between(start: &str, end: &str) -> f64 {
    // Very simple: parse YYYY-MM-DD, return difference + 1
    let parse = |s: &str| -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
    };
    if let (Some(s), Some(e)) = (parse(start), parse(end)) {
        ((e - s).num_days() + 1).max(1) as f64
    } else {
        1.0
    }
}

fn parse_leave_type(s: &str) -> LeaveType {
    match s {
        "SICK" => LeaveType::Sick,
        "CASUAL" => LeaveType::Casual,
        "MATERNITY" => LeaveType::Maternity,
        "PATERNITY" => LeaveType::Paternity,
        "COMPENSATORY" => LeaveType::Compensatory,
        "UNPAID" => LeaveType::Unpaid,
        _ => LeaveType::Annual,
    }
}

fn parse_duration(s: &str) -> LeaveDuration {
    match s {
        "FIRST_HALF" => LeaveDuration::FirstHalf,
        "SECOND_HALF" => LeaveDuration::SecondHalf,
        _ => LeaveDuration::FullDay,
    }
}

// ─── Employee: my leaves ───────────────────────────────────

pub async fn get_my_leaves(
    State(state): State<AppState>,
    user_id: UserId,
    Query(params): Query<PageParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10);

    let requests = state.leave_requests.lock().unwrap();
    let mut filtered: Vec<LeaveRequest> = requests
        .iter()
        .filter(|r| r.employee_id == user_id.0)
        .filter(|r| {
            params.status.as_ref().map_or(true, |s| format!("{:?}", r.status).to_uppercase() == s.to_uppercase())
        })
        .filter(|r| {
            params.leave_type.as_ref().map_or(true, |lt| format!("{:?}", r.leave_type).to_uppercase() == lt.to_uppercase())
        })
        .cloned()
        .collect();
    filtered.sort_by(|a, b| b.applied_on.cmp(&a.applied_on));

    let (items, total) = paginate(filtered, page, page_size);
    Json(json!(ApiResponse::ok(PaginatedResponse::new(items, total, page, page_size), "OK"))).into_response()
}

// ─── Employee: single leave ────────────────────────────────

pub async fn get_leave_by_id(
    State(state): State<AppState>,
    _user_id: UserId,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let requests = state.leave_requests.lock().unwrap();
    match requests.iter().find(|r| r.id == id) {
        Some(r) => Json(json!(ApiResponse::ok(r.clone(), "OK"))).into_response(),
        None => err(StatusCode::NOT_FOUND, "Leave request not found").into_response(),
    }
}

// ─── Employee: cancel leave ────────────────────────────────

pub async fn cancel_leave(
    State(state): State<AppState>,
    user_id: UserId,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut requests = state.leave_requests.lock().unwrap();
    match requests.iter_mut().find(|r| r.id == id && r.employee_id == user_id.0) {
        Some(r) => {
            r.status = LeaveStatus::Cancelled;
            let cloned = r.clone();
            Json(json!(ApiResponse::ok(cloned, "Leave cancelled"))).into_response()
        }
        None => err(StatusCode::NOT_FOUND, "Leave request not found").into_response(),
    }
}

// ─── Employee: leave balance ───────────────────────────────

pub async fn get_balance(
    State(state): State<AppState>,
    user_id: UserId,
) -> impl IntoResponse {
    let balances = state.leave_balances.lock().unwrap();
    let result: Vec<LeaveBalance> = balances
        .iter()
        .filter(|b| b.employee_id == user_id.0)
        .cloned()
        .collect();
    Json(json!(ApiResponse::ok(result, "OK"))).into_response()
}

// ─── Manager: pending approvals ────────────────────────────

pub async fn get_pending_approvals(
    State(state): State<AppState>,
    _user_id: UserId,
    Query(params): Query<PageParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10);

    let requests = state.leave_requests.lock().unwrap();
    let filtered: Vec<LeaveRequest> = requests
        .iter()
        .filter(|r| matches!(r.status, LeaveStatus::Pending))
        .cloned()
        .collect();
    let (items, total) = paginate(filtered, page, page_size);
    Json(json!(ApiResponse::ok(PaginatedResponse::new(items, total, page, page_size), "OK"))).into_response()
}

// ─── Manager: approve / reject ─────────────────────────────

pub async fn approve_or_reject(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<LeaveActionPayload>,
) -> impl IntoResponse {
    let users = state.users.lock().unwrap().clone();
    let approver = users.iter().find(|u| u.id == user_id.0).cloned();

    let mut requests = state.leave_requests.lock().unwrap();
    match requests.iter_mut().find(|r| r.id == payload.leave_request_id) {
        Some(r) => {
            r.status = if payload.action.to_uppercase() == "APPROVE" {
                LeaveStatus::Approved
            } else {
                LeaveStatus::Rejected
            };
            if let Some(ref a) = approver {
                r.approved_by = Some(a.id.clone());
                r.approver_name = Some(format!("{} {}", a.first_name, a.last_name));
            }
            r.approved_on = Some(today());
            r.rejection_reason = payload.comments.clone();
            let cloned = r.clone();
            let msg = format!("Leave request {}d", payload.action.to_lowercase());
            Json(json!(ApiResponse::ok(cloned, msg))).into_response()
        }
        None => err(StatusCode::NOT_FOUND, "Leave request not found").into_response(),
    }
}

// ─── Manager: team leaves ──────────────────────────────────

pub async fn get_team_leaves(
    State(state): State<AppState>,
    _user_id: UserId,
    Query(params): Query<PageParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10);
    let requests = state.leave_requests.lock().unwrap();
    let all: Vec<LeaveRequest> = requests.iter().cloned().collect();
    let (items, total) = paginate(all, page, page_size);
    Json(json!(ApiResponse::ok(PaginatedResponse::new(items, total, page, page_size), "OK"))).into_response()
}

// ─── Calendar ─────────────────────────────────────────────

pub async fn get_calendar(
    State(state): State<AppState>,
    _user_id: UserId,
    Query(params): Query<PageParams>,
) -> impl IntoResponse {
    let month = params.month.unwrap_or(chrono::Utc::now().month());
    let year = params.year.unwrap_or(chrono::Utc::now().year());
    let month_str = format!("{:04}-{:02}", year, month);

    let requests = state.leave_requests.lock().unwrap();
    let entries: Vec<serde_json::Value> = requests
        .iter()
        .filter(|r| r.start_date.starts_with(&month_str) && matches!(r.status, LeaveStatus::Approved))
        .map(|r| json!({
            "employeeId": r.employee_id,
            "employeeName": r.employee_name,
            "department": r.department,
            "date": r.start_date,
            "leaveType": r.leave_type,
            "duration": r.duration,
            "status": r.status,
        }))
        .collect();
    Json(json!(ApiResponse::ok(entries, "OK"))).into_response()
}

// ─── Holidays ─────────────────────────────────────────────

pub async fn get_holidays(
    State(state): State<AppState>,
    _user_id: UserId,
    Query(params): Query<PageParams>,
) -> impl IntoResponse {
    let holidays = state.holidays.lock().unwrap();
    let filtered: Vec<Holiday> = holidays.iter().filter(|h| {
        params.year.map_or(true, |y| h.date.starts_with(&format!("{}", y)))
    }).cloned().collect();
    Json(json!(ApiResponse::ok(filtered, "OK"))).into_response()
}

pub async fn create_holiday(
    State(state): State<AppState>,
    _user_id: UserId,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let holiday = Holiday {
        id: format!("H{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("000")),
        name: payload.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        date: payload.get("date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        is_optional: payload.get("isOptional").and_then(|v| v.as_bool()).unwrap_or(false),
        description: payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };
    state.holidays.lock().unwrap().push(holiday.clone());
    (StatusCode::CREATED, Json(json!(ApiResponse::ok(holiday, "Holiday created")))).into_response()
}

pub async fn delete_holiday(
    State(state): State<AppState>,
    _user_id: UserId,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut holidays = state.holidays.lock().unwrap();
    let len_before = holidays.len();
    holidays.retain(|h| h.id != id);
    if holidays.len() < len_before {
        Json(json!(ApiResponse::ok(json!(null), "Holiday deleted"))).into_response()
    } else {
        err(StatusCode::NOT_FOUND, "Holiday not found").into_response()
    }
}

// ─── HR Admin: all requests ────────────────────────────────

pub async fn get_all_leaves(
    State(state): State<AppState>,
    _user_id: UserId,
    Query(params): Query<PageParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10);

    let requests = state.leave_requests.lock().unwrap();
    let filtered: Vec<LeaveRequest> = requests.iter()
        .filter(|r| params.status.as_ref().map_or(true, |s| format!("{:?}", r.status).to_uppercase() == s.to_uppercase()))
        .filter(|r| params.leave_type.as_ref().map_or(true, |lt| format!("{:?}", r.leave_type).to_uppercase() == lt.to_uppercase()))
        .filter(|r| params.department.as_ref().map_or(true, |d| r.department.eq_ignore_ascii_case(d)))
        .cloned()
        .collect();
    let (items, total) = paginate(filtered, page, page_size);
    Json(json!(ApiResponse::ok(PaginatedResponse::new(items, total, page, page_size), "OK"))).into_response()
}

// ─── HR Admin: policies ────────────────────────────────────

pub async fn get_policies(State(state): State<AppState>, _user_id: UserId) -> impl IntoResponse {
    let policies = state.leave_policies.lock().unwrap().clone();
    Json(json!(ApiResponse::ok(policies, "OK"))).into_response()
}

pub async fn update_policy(
    State(state): State<AppState>,
    _user_id: UserId,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut policies = state.leave_policies.lock().unwrap();
    match policies.iter_mut().find(|p| p.id == id) {
        Some(p) => {
            if let Some(v) = payload.get("totalDaysPerYear").and_then(|v| v.as_i64()) {
                p.total_days_per_year = v as i32;
            }
            if let Some(v) = payload.get("maxConsecutiveDays").and_then(|v| v.as_i64()) {
                p.max_consecutive_days = v as i32;
            }
            if let Some(v) = payload.get("carryForwardLimit").and_then(|v| v.as_i64()) {
                p.carry_forward_limit = v as i32;
            }
            if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
                p.description = v.to_string();
            }
            if let Some(v) = payload.get("isActive").and_then(|v| v.as_bool()) {
                p.is_active = v;
            }
            let cloned = p.clone();
            Json(json!(ApiResponse::ok(cloned, "Policy updated"))).into_response()
        }
        None => err(StatusCode::NOT_FOUND, "Policy not found").into_response(),
    }
}

// Needed for chrono month/year in PageParams
use chrono::Datelike;
