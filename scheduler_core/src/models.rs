use crate::error::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: String,
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
        DateTime::parse_from_rfc3339(&self.schedule.to_rfc3339()).map_err(|_| {
            Error::ValidationError("Invalid datetime format. Use ISO 8601 format".into())
        })?;
        Ok(())
    }
}
