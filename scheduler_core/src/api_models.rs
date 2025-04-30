use crate::models::JobType;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize, de::Visitor};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct JobCreate {
    pub schedule_type: JobType,
    pub schedule: DateTime<Utc>,
    pub payload: Value,
    pub max_retries: u32,
    pub template_id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JobUpdate {
    pub schedule_type: Option<JobType>,
    pub schedule: Option<DateTime<Utc>>,
    pub payload: Option<Value>,
    pub max_retries: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateCreate {
    pub cron_pattern: String,
    pub payload_template: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateUpdate {
    pub cron_pattern: Option<String>,
    pub payload_template: Option<Value>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub message: String,
    pub job_id: String,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub message: String,
    pub template_id: i32,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub message: String,
}
