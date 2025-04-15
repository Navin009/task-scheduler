use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringJob {
    pub id: String,
    pub schedule: String, // e.g., cron expression
    pub job_details: Job,
}