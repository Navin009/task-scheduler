use crate::error::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: String,
    pub schedule_type: JobType,
    pub schedule: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retries: i32,
    pub max_retries: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Template {
    pub id: Uuid,
    pub cron: Option<String>,
    pub payload: serde_json::Value,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "job_status")]
#[sqlx(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "job_type")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    #[serde(rename = "one_time")]
    OneTime,
    #[serde(rename = "recurring")]
    Recurring,
    #[serde(rename = "polling")]
    Polling,
}

impl TryFrom<&str> for JobType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "one_time" => Ok(JobType::OneTime),
            "recurring" => Ok(JobType::Recurring),
            "polling" => Ok(JobType::Polling),
            _ => Err(Error::ValidationError(format!(
                "Invalid schedule type: {}",
                value
            ))),
        }
    }
}

impl From<JobType> for String {
    fn from(value: JobType) -> Self {
        match value {
            JobType::OneTime => "one_time".to_string(),
            JobType::Recurring => "recurring".to_string(),
            JobType::Polling => "polling".to_string(),
        }
    }
}

impl Job {
    pub fn validate(&self) -> Result<(), Error> {
        match self.schedule_type {
            JobType::Recurring => {
                let now = Utc::now();
                cron_parser::parse(&self.schedule.to_rfc3339(), &now)
                    .map_err(|e| Error::ValidationError(e.to_string()))?;
            }
            JobType::OneTime => {
                DateTime::parse_from_rfc3339(&self.schedule.to_rfc3339()).map_err(|_| {
                    Error::ValidationError("Invalid datetime format. Use ISO 8601 format".into())
                })?;
            }
            JobType::Polling => {
                let polling_config: serde_json::Value =
                    serde_json::from_str(&self.schedule.to_rfc3339()).map_err(|_| {
                        Error::ValidationError("Invalid polling config format".into())
                    })?;

                if !polling_config.is_object() {
                    return Err(Error::ValidationError(
                        "Polling config must be a JSON object".into(),
                    ));
                }

                let interval = polling_config
                    .get("interval")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        Error::ValidationError(
                            "Missing or invalid interval in polling config".into(),
                        )
                    })?;

                let max_attempts = polling_config
                    .get("max_attempts")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        Error::ValidationError(
                            "Missing or invalid max_attempts in polling config".into(),
                        )
                    })?;

                if interval == 0 || max_attempts == 0 {
                    return Err(Error::ValidationError(
                        "Interval and max_attempts must be greater than 0".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}
