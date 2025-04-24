use crate::database::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub priority: i32,
    pub scheduled_at: String,
    pub parent_job_id: Option<String>,
    pub max_attempts: i32,
    pub attempts: i32,
    pub payload: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JobType {
    OneTime,
    Recurring,
    Polling,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
}

pub struct TaskManager {
    db: Database,
}

impl TaskManager {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_one_time_job(
        &self,
        scheduled_at: String,
        priority: i32,
        payload: HashMap<String, String>,
    ) -> Result<String> {
        let mut job_data = HashMap::new();
        job_data.insert("job_type", "one_time".to_string());
        job_data.insert("status", "pending".to_string());
        job_data.insert("priority", priority.to_string());
        job_data.insert("scheduled_at", scheduled_at);
        job_data.insert("max_attempts", "1".to_string());
        job_data.insert("attempts", "0".to_string());
        job_data.insert("payload", serde_json::to_string(&payload)?);

        self.db.create_job(&job_data).await
    }

    pub async fn create_recurring_job(
        &self,
        parent_job_id: String,
        scheduled_at: String,
        priority: i32,
        payload: HashMap<String, String>,
    ) -> Result<String> {
        let mut job_data = HashMap::new();
        job_data.insert("job_type", "recurring".to_string());
        job_data.insert("status", "pending".to_string());
        job_data.insert("priority", priority.to_string());
        job_data.insert("scheduled_at", scheduled_at);
        job_data.insert("parent_job_id", parent_job_id);
        job_data.insert("max_attempts", "1".to_string());
        job_data.insert("attempts", "0".to_string());
        job_data.insert("payload", serde_json::to_string(&payload)?);

        self.db.create_job(&job_data).await
    }

    pub async fn create_polling_job(
        &self,
        scheduled_at: String,
        priority: i32,
        max_attempts: i32,
        payload: HashMap<String, String>,
    ) -> Result<String> {
        let mut job_data = HashMap::new();
        job_data.insert("job_type", "polling".to_string());
        job_data.insert("status", "pending".to_string());
        job_data.insert("priority", priority.to_string());
        job_data.insert("scheduled_at", scheduled_at);
        job_data.insert("max_attempts", max_attempts.to_string());
        job_data.insert("attempts", "0".to_string());
        job_data.insert("payload", serde_json::to_string(&payload)?);

        self.db.create_job(&job_data).await
    }

    pub async fn get_job(&self, id: &str) -> Result<Option<Job>> {
        let job_data = self.db.get_job(id).await?;
        Ok(job_data.map(|data| Job {
            id: data.get("id").unwrap().clone(),
            job_type: match data.get("job_type").unwrap().as_str() {
                "one_time" => JobType::OneTime,
                "recurring" => JobType::Recurring,
                "polling" => JobType::Polling,
                _ => panic!("Invalid job type"),
            },
            status: match data.get("status").unwrap().as_str() {
                "pending" => JobStatus::Pending,
                "queued" => JobStatus::Queued,
                "running" => JobStatus::Running,
                "completed" => JobStatus::Completed,
                "failed" => JobStatus::Failed,
                _ => panic!("Invalid job status"),
            },
            priority: data.get("priority").unwrap().parse().unwrap(),
            scheduled_at: data.get("scheduled_at").unwrap().clone(),
            parent_job_id: data.get("parent_job_id").map(|s| s.clone()),
            max_attempts: data.get("max_attempts").unwrap().parse().unwrap(),
            attempts: data.get("attempts").unwrap().parse().unwrap(),
            payload: serde_json::from_str(data.get("payload").unwrap()).unwrap(),
        }))
    }

    pub async fn update_job_status(&self, id: &str, status: JobStatus) -> Result<bool> {
        let mut updates = HashMap::new();
        updates.insert("status", status.to_string());
        self.db.update_job(id, &updates).await
    }

    pub async fn increment_job_attempts(&self, id: &str) -> Result<bool> {
        let mut updates = HashMap::new();
        updates.insert("attempts", "attempts + 1".to_string());
        self.db.update_job(id, &updates).await
    }

    pub async fn get_due_jobs(&self, limit: i64) -> Result<Vec<Job>> {
        let job_types = vec!["one_time", "recurring", "polling"];
        let job_data = self.db.get_due_jobs(limit, &job_types).await?;

        Ok(job_data
            .into_iter()
            .map(|data| Job {
                id: data.get("id").unwrap().clone(),
                job_type: match data.get("job_type").unwrap().as_str() {
                    "one_time" => JobType::OneTime,
                    "recurring" => JobType::Recurring,
                    "polling" => JobType::Polling,
                    _ => panic!("Invalid job type"),
                },
                status: match data.get("status").unwrap().as_str() {
                    "pending" => JobStatus::Pending,
                    "queued" => JobStatus::Queued,
                    "running" => JobStatus::Running,
                    "completed" => JobStatus::Completed,
                    "failed" => JobStatus::Failed,
                    _ => panic!("Invalid job status"),
                },
                priority: data.get("priority").unwrap().parse().unwrap(),
                scheduled_at: data.get("scheduled_at").unwrap().clone(),
                parent_job_id: data.get("parent_job_id").map(|s| s.clone()),
                max_attempts: data.get("max_attempts").unwrap().parse().unwrap(),
                attempts: data.get("attempts").unwrap().parse().unwrap(),
                payload: serde_json::from_str(data.get("payload").unwrap()).unwrap(),
            })
            .collect())
    }
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        match self {
            JobStatus::Pending => "pending".to_string(),
            JobStatus::Queued => "queued".to_string(),
            JobStatus::Running => "running".to_string(),
            JobStatus::Completed => "completed".to_string(),
            JobStatus::Failed => "failed".to_string(),
        }
    }
}
