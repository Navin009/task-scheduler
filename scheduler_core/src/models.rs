use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub schedule_type: ScheduleType,
    pub schedule: String,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retries: u32,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub cron_pattern: String,
    pub payload_template: serde_json::Value,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScheduleType {
    Immediate,
    Cron,
    Interval,
}

impl Job {
    pub fn validate(&self) -> Result<(), Error> {
        match self.schedule_type {
            ScheduleType::Cron => {
                let now = Utc::now();
                cron_parser::parse(&self.schedule, &now)
                    .map_err(|e| Error::ValidationError(e.to_string()))?;
            }
            ScheduleType::Interval => {
                self.schedule.parse::<u64>()
                    .map_err(|_| Error::ValidationError("Invalid interval format".into()))?;
            }
            ScheduleType::Immediate => {}
        }
        Ok(())
    }
}
