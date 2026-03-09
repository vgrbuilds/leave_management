use serde::{Deserialize, Serialize};

// ─── Enums ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Employee,
    Manager,
    HrAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaveType {
    Annual,
    Sick,
    Casual,
    Maternity,
    Paternity,
    Compensatory,
    Unpaid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaveStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaveDuration {
    FullDay,
    FirstHalf,
    SecondHalf,
}

// ─── Data Structs ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub employee_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub department: String,
    pub designation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager_name: Option<String>,
    pub joining_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emergency_contact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emergency_phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveRequest {
    pub id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub department: String,
    pub leave_type: LeaveType,
    pub start_date: String,
    pub end_date: String,
    pub duration: LeaveDuration,
    pub total_days: f64,
    pub reason: String,
    pub status: LeaveStatus,
    pub applied_on: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approver_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveBalance {
    pub employee_id: String,
    pub leave_type: LeaveType,
    pub leave_type_name: String,
    pub total: f64,
    pub used: f64,
    pub pending: f64,
    pub available: f64,
    pub carry_forward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeavePolicy {
    pub id: String,
    pub leave_type: LeaveType,
    pub leave_type_name: String,
    pub total_days_per_year: i32,
    pub max_consecutive_days: i32,
    pub carry_forward_limit: i32,
    pub min_service_days_required: i32,
    pub requires_attachment: bool,
    pub requires_approval: bool,
    pub description: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Holiday {
    pub id: String,
    pub name: String,
    pub date: String,
    pub is_optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ─── API Response Wrappers ────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    pub message: String,
    pub timestamp: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data,
            message: message.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total_items: usize,
    pub current_page: usize,
    pub total_pages: usize,
    pub page_size: usize,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total_items: usize, page: usize, page_size: usize) -> Self {
        let total_pages = if page_size == 0 { 0 } else { (total_items + page_size - 1) / page_size };
        Self { items, total_items, current_page: page, total_pages, page_size }
    }
}

// ─── Auth ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterPayload {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub employee_id: String,
    pub department: String,
    pub designation: String,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: PublicUser,
}

/// Safe public view of a user — no password field.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUser {
    pub id: String,
    pub employee_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role: UserRole,
    pub department: String,
    pub designation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager_name: Option<String>,
    pub joining_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emergency_contact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emergency_phone: Option<String>,
}

impl From<User> for PublicUser {
    fn from(u: User) -> Self {
        Self {
            id: u.id, employee_id: u.employee_id, first_name: u.first_name,
            last_name: u.last_name, email: u.email, role: u.role,
            department: u.department, designation: u.designation,
            manager_id: u.manager_id, manager_name: u.manager_name,
            joining_date: u.joining_date, avatar_url: u.avatar_url,
            phone: u.phone, is_active: u.is_active, address: u.address,
            emergency_contact: u.emergency_contact, emergency_phone: u.emergency_phone,
        }
    }
}

// ─── Dashboard ────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_employees: usize,
    pub pending_requests: usize,
    pub approved_today: usize,
    pub on_leave_today: usize,
    pub upcoming_holidays: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceReport {
    pub department: String,
    pub total_employees: usize,
    pub average_leaves_taken: f64,
    pub leave_utilization_percent: f64,
    pub excess_leave_count: usize,
    pub pending_approvals: usize,
}

// ─── Other Payloads ───────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveActionPayload {
    pub leave_request_id: String,
    pub action: String,
    pub comments: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordPayload {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshPayload {
    pub refresh_token: String,
}
