use crate::error::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: String,
    pub schedule_type: ScheduleType,
    pub schedule: String,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retries: i32,
    pub max_retries: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Template {
    pub id: String,
    pub cron_pattern: String,
    pub payload_template: serde_json::Value,
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
#[sqlx(type_name = "schedule_type")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ScheduleType {
    #[serde(rename = "one_time")]
    OneTime,
    #[serde(rename = "recurring")]
    Recurring,
    #[serde(rename = "polling")]
    Polling,
}

impl TryFrom<&str> for ScheduleType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "one_time" => Ok(ScheduleType::OneTime),
            "recurring" => Ok(ScheduleType::Recurring),
            "polling" => Ok(ScheduleType::Polling),
            _ => Err(Error::ValidationError(format!(
                "Invalid schedule type: {}",
                value
            ))),
        }
    }
}

impl From<ScheduleType> for String {
    fn from(value: ScheduleType) -> Self {
        match value {
            ScheduleType::OneTime => "one_time".to_string(),
            ScheduleType::Recurring => "recurring".to_string(),
            ScheduleType::Polling => "polling".to_string(),
        }
    }
}

impl Job {
    pub fn validate(&self) -> Result<(), Error> {
        match self.schedule_type {
            ScheduleType::Recurring => {
                let now = Utc::now();
                cron_parser::parse(&self.schedule, &now)
                    .map_err(|e| Error::ValidationError(e.to_string()))?;
            }
            ScheduleType::OneTime => {
                DateTime::parse_from_rfc3339(&self.schedule).map_err(|_| {
                    Error::ValidationError("Invalid datetime format. Use ISO 8601 format".into())
                })?;
            }
            ScheduleType::Polling => {
                let polling_config: serde_json::Value = serde_json::from_str(&self.schedule)
                    .map_err(|_| Error::ValidationError("Invalid polling config format".into()))?;

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
