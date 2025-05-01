use crate::models::JobType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct JobCreate {
    pub schedule_type: JobType,
    pub cron: Option<String>,
    pub interval: Option<u32>,
    pub schedule_at: Option<DateTime<Utc>>,
    pub payload: Value,
    pub max_retries: Option<u32>,
    pub template_id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JobUpdate {
    pub schedule_type: Option<JobType>,
    pub cron: Option<String>,
    pub interval: Option<u32>,
    pub schedule_at: Option<DateTime<Utc>>,
    pub payload: Option<Value>,
    pub max_retries: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateCreate {
    pub cron: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateUpdate {
    pub cron: Option<String>,
    pub payload: Option<Value>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub message: String,
    pub job_id: Uuid,
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
