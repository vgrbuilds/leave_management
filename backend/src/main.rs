mod handlers;
mod middleware;
mod models;
mod state;

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use handlers::{auth, leaves, users};
use state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::load();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // ─── Auth ───────────────────────────────
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/refresh", post(auth::refresh_token))
        // ─── Leaves ─────────────────────────────
        .route("/api/leaves/apply", post(leaves::apply_leave))
        .route("/api/leaves/my-leaves", get(leaves::get_my_leaves))
        .route("/api/leaves/balance", get(leaves::get_balance))
        .route("/api/leaves/pending-approvals", get(leaves::get_pending_approvals))
        .route("/api/leaves/action", post(leaves::approve_or_reject))
        .route("/api/leaves/team-leaves", get(leaves::get_team_leaves))
        .route("/api/leaves/calendar", get(leaves::get_calendar))
        .route("/api/leaves/holidays", get(leaves::get_holidays))
        .route("/api/leaves/holidays", post(leaves::create_holiday))
        .route("/api/leaves/holidays/{id}", delete(leaves::delete_holiday))
        .route("/api/leaves/all", get(leaves::get_all_leaves))
        .route("/api/leaves/policies", get(leaves::get_policies))
        .route("/api/leaves/policies/{id}", put(leaves::update_policy))
        .route("/api/leaves/{id}", get(leaves::get_leave_by_id))
        .route("/api/leaves/{id}/cancel", patch(leaves::cancel_leave))
        // ─── Users ──────────────────────────────
        .route("/api/users/profile", get(users::get_profile))
        .route("/api/users/profile", put(users::update_profile))
        .route("/api/users/change-password", post(users::change_password))
        .route("/api/users/dashboard-stats", get(users::get_dashboard_stats))
        .route("/api/users/compliance-report", get(users::get_compliance_report))
        .route("/api/users/team", get(users::get_team))
        .route("/api/users", get(users::get_all_users))
        .route("/api/users/{id}", get(users::get_user_by_id))
        .route("/api/users/{id}", put(users::update_user))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:8080";
    println!("🚀 Leave Management API running at http://{}/api", addr);
    println!("   Login: john.doe@company.com / password123 (Employee)");
    println!("   Login: jane.smith@company.com / password123 (Manager)");
    println!("   Login: sarah.johnson@company.com / password123 (HR Admin)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
