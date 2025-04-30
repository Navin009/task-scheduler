use crate::{JobStatus, JobType, db::Database};
use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub priority: i32,
    pub scheduled_at: String,
    pub parent_job_id: Option<String>,
    pub max_retries: i32,
    pub retries: i32,
    pub payload: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TaskManager {
    db: Database,
}

impl TaskManager {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_one_time_job(
        &self,
        scheduled_at: NaiveDateTime,
        priority: i32,
        payload: HashMap<String, String>,
    ) -> Result<String> {
        let job_data = crate::db::JobData {
            job_type: JobType::OneTime,
            status: JobStatus::Pending,
            priority,
            scheduled_at: format!("{}::timestamp with time zone", scheduled_at),
            parent_job_id: None,
            max_retries: 3,
            retries: 0,
            payload: to_value(payload)?,
        };

        self.db.create_job(job_data).await
    }

    pub async fn create_recurring_job(
        &self,
        parent_job_id: String,
        scheduled_at: NaiveDateTime,
        priority: i32,
        payload: HashMap<String, String>,
    ) -> Result<String> {
        let job_data = crate::db::JobData {
            job_type: JobType::Recurring,
            status: JobStatus::Pending,
            priority,
            scheduled_at: format!("{}::timestamp with time zone", scheduled_at),
            parent_job_id: Some(parent_job_id),
            max_retries: 3,
            retries: 0,
            payload: to_value(payload)?,
        };

        self.db.create_job(job_data).await
    }

    pub async fn create_polling_job(
        &self,
        scheduled_at: NaiveDateTime,
        priority: i32,
        max_retries: i32,
        payload: HashMap<String, String>,
    ) -> Result<String> {
        let job_data = crate::db::JobData {
            job_type: JobType::Polling,
            status: JobStatus::Pending,
            priority,
            scheduled_at: format!("{}::timestamp with time zone", scheduled_at),
            parent_job_id: None,
            max_retries,
            retries: 0,
            payload: to_value(payload)?,
        };

        self.db.create_job(job_data).await
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
                "running" => JobStatus::Running,
                "completed" => JobStatus::Completed,
                "failed" => JobStatus::Failed,
                "retrying" => JobStatus::Retrying,
                _ => panic!("Invalid job status"),
            },
            priority: data.get("priority").unwrap().parse().unwrap(),
            scheduled_at: data.get("scheduled_at").unwrap().clone(),
            parent_job_id: data.get("parent_job_id").map(|s| s.clone()),
            max_retries: data.get("max_retries").unwrap().parse().unwrap(),
            retries: data.get("retries").unwrap().parse().unwrap(),
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
        updates.insert("retries", "retries + 1".to_string());
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
                    "running" => JobStatus::Running,
                    "completed" => JobStatus::Completed,
                    "failed" => JobStatus::Failed,
                    "retrying" => JobStatus::Retrying,
                    _ => panic!("Invalid job status"),
                },
                priority: data.get("priority").unwrap().parse().unwrap(),
                scheduled_at: data.get("scheduled_at").unwrap().clone(),
                parent_job_id: data.get("parent_job_id").map(|s| s.clone()),
                max_retries: data.get("max_retries").unwrap().parse().unwrap(),
                retries: data.get("retries").unwrap().parse().unwrap(),
                payload: serde_json::from_str(data.get("payload").unwrap()).unwrap(),
            })
            .collect())
    }

    pub async fn get_jobs_by_status(&self, status: JobStatus) -> Result<Vec<Job>> {
        let job_data = self.db.get_jobs_by_status(&status.to_string()).await?;

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
                    "running" => JobStatus::Running,
                    "completed" => JobStatus::Completed,
                    "failed" => JobStatus::Failed,
                    "retrying" => JobStatus::Retrying,
                    _ => panic!("Invalid job status"),
                },
                priority: data.get("priority").unwrap().parse().unwrap(),
                scheduled_at: data.get("scheduled_at").unwrap().clone(),
                parent_job_id: data.get("parent_job_id").map(|s| s.clone()),
                max_retries: data.get("max_retries").unwrap().parse().unwrap(),
                retries: data.get("retries").unwrap().parse().unwrap(),
                payload: serde_json::from_str(data.get("payload").unwrap()).unwrap(),
            })
            .collect())
    }

    pub async fn get_jobs_older_than(
        &self,
        cutoff_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Job>> {
        let job_data = self
            .db
            .get_jobs_older_than(&cutoff_time.to_rfc3339())
            .await?;
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
                    "running" => JobStatus::Running,
                    "completed" => JobStatus::Completed,
                    "failed" => JobStatus::Failed,
                    "retrying" => JobStatus::Retrying,
                    _ => panic!("Invalid job status"),
                },
                priority: data.get("priority").unwrap().parse().unwrap(),
                scheduled_at: data.get("scheduled_at").unwrap().clone(),
                parent_job_id: data.get("parent_job_id").map(|s| s.clone()),
                max_retries: data.get("max_retries").unwrap().parse().unwrap(),
                retries: data.get("retries").unwrap().parse().unwrap(),
                payload: serde_json::from_str(data.get("payload").unwrap()).unwrap(),
            })
            .collect())
    }

    pub async fn get_jobs_by_status_and_time(
        &self,
        status: JobStatus,
        cutoff_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Job>> {
        let job_data = self
            .db
            .get_jobs_by_status_and_time(&status.to_string(), &cutoff_time.to_rfc3339())
            .await?;
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
                    "running" => JobStatus::Running,
                    "completed" => JobStatus::Completed,
                    "failed" => JobStatus::Failed,
                    "retrying" => JobStatus::Retrying,
                    _ => panic!("Invalid job status"),
                },
                priority: data.get("priority").unwrap().parse().unwrap(),
                scheduled_at: data.get("scheduled_at").unwrap().clone(),
                parent_job_id: data.get("parent_job_id").map(|s| s.clone()),
                max_retries: data.get("max_retries").unwrap().parse().unwrap(),
                retries: data.get("retries").unwrap().parse().unwrap(),
                payload: serde_json::from_str(data.get("payload").unwrap()).unwrap(),
            })
            .collect())
    }

    pub async fn move_to_dead_letter_queue(&self, job_id: &str, queue_name: &str) -> Result<bool> {
        let mut updates = HashMap::new();
        updates.insert("status", JobStatus::Failed.to_string());
        updates.insert("dead_letter_queue", queue_name.to_string());
        self.db.update_job(job_id, &updates).await
    }

    pub async fn archive_job(&self, job_id: &str) -> Result<bool> {
        let mut updates: HashMap<&str, String> = HashMap::new();
        updates.insert("archived", "true".to_string());
        self.db.update_job(job_id, &updates).await
    }
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        match self {
            JobStatus::Pending => "pending".to_string(),
            JobStatus::Running => "running".to_string(),
            JobStatus::Completed => "completed".to_string(),
            JobStatus::Failed => "failed".to_string(),
            JobStatus::Retrying => "retrying".to_string(),
        }
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::OneTime => write!(f, "one_time"),
            JobType::Recurring => write!(f, "recurring"),
            JobType::Polling => write!(f, "polling"),
        }
    }
}
