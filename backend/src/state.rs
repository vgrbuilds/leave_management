use std::sync::{Arc, Mutex};
use crate::models::{User, LeaveRequest, LeaveBalance, Holiday, LeavePolicy};

/// All application data lives here. Loaded from JSON on startup,
/// mutated in-memory for the lifetime of the process.
#[derive(Clone)]
pub struct AppState {
    pub users: Arc<Mutex<Vec<User>>>,
    pub leave_requests: Arc<Mutex<Vec<LeaveRequest>>>,
    pub leave_balances: Arc<Mutex<Vec<LeaveBalance>>>,
    pub holidays: Arc<Mutex<Vec<Holiday>>>,
    pub leave_policies: Arc<Mutex<Vec<LeavePolicy>>>,
    pub jwt_secret: String,
}

impl AppState {
    pub fn load() -> Self {
        let data_dir = Self::data_dir();

        let users: Vec<User> = Self::load_json(&format!("{}/users.json", data_dir));
        let leave_requests: Vec<LeaveRequest> =
            Self::load_json(&format!("{}/leave_requests.json", data_dir));
        let leave_balances: Vec<LeaveBalance> =
            Self::load_json(&format!("{}/leave_balances.json", data_dir));
        let holidays: Vec<Holiday> = Self::load_json(&format!("{}/holidays.json", data_dir));
        let leave_policies: Vec<LeavePolicy> =
            Self::load_json(&format!("{}/leave_policies.json", data_dir));

        println!("✅ Loaded {} users", users.len());
        println!("✅ Loaded {} leave requests", leave_requests.len());
        println!("✅ Loaded {} leave balances", leave_balances.len());
        println!("✅ Loaded {} holidays", holidays.len());
        println!("✅ Loaded {} leave policies", leave_policies.len());

        Self {
            users: Arc::new(Mutex::new(users)),
            leave_requests: Arc::new(Mutex::new(leave_requests)),
            leave_balances: Arc::new(Mutex::new(leave_balances)),
            holidays: Arc::new(Mutex::new(holidays)),
            leave_policies: Arc::new(Mutex::new(leave_policies)),
            jwt_secret: "lms_jwt_secret_mvp_2026".to_string(),
        }
    }

    fn data_dir() -> String {
        // Try CWD/data first (for `cargo run` from workspace root),
        // then the directory of the executable.
        let candidates = vec!["data", "../backend/data"];
        for candidate in candidates {
            if std::path::Path::new(candidate).exists() {
                return candidate.to_string();
            }
        }
        "data".to_string()
    }

    fn load_json<T: serde::de::DeserializeOwned>(path: &str) -> Vec<T> {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!("⚠️  Failed to parse {}: {}", path, e);
                vec![]
            }),
            Err(e) => {
                eprintln!("⚠️  Could not read {}: {}", path, e);
                vec![]
            }
        }
    }
}
