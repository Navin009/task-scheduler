use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "task_status", rename_all = "lowercase")] // For PostgreSQL enum mapping
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retry, // Added a retry state
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Task {
    #[sqlx(try_from = "uuid::Uuid")] // Ensure correct mapping if DB type is UUID
    pub id: Uuid,
    pub name: String,
    #[sqlx(rename = "schedule_str")] // Use a different name in DB if needed
    pub schedule: String, // e.g., cron string "* * * * * *" or "every 5m"
    pub payload: Value,   // Flexible JSON payload
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub next_run_time: Option<DateTime<Utc>>,
    pub last_run_time: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub max_retries: i32,
    pub current_retries: i32,
}

impl Task {
    pub fn new(name: String, schedule: String, payload: Value, max_retries: i32) -> Self {
        let now = Utc::now();
        Task {
            id: Uuid::new_v4(),
            name,
            schedule,
            payload,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            next_run_time: None, // Should be calculated based on schedule
            last_run_time: None,
            last_error: None,
            max_retries,
            current_retries: 0,
        }
    }

    // Placeholder for schedule parsing logic
    pub fn calculate_next_run(&self, _now: DateTime<Utc>) -> Result<Option<DateTime<Utc>>> {
        // TODO: Implement actual cron or interval parsing logic here
        // For now, just return None or a fixed time for testing
        // Example using a hypothetical parser:
        // parse_schedule(&self.schedule, now).map_err(|e| SchedulerError::InvalidSchedule(e.to_string()))
        Ok(None) // Replace with actual calculation
    }
}
