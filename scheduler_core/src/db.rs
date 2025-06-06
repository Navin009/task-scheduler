use crate::models::Template;
use crate::{JobStatus, JobType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Column, Row};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

#[derive(Debug)]
pub struct JobData {
    pub name: Option<String>,
    pub status: JobStatus,
    pub parent_job_id: Option<Uuid>,
    pub description: Option<String>,
    pub priority: i32,
    pub max_retries: i32,
    pub retries: i32,
    pub payload: Value,
    pub interval: Option<u32>,
    pub cron: Option<String>,
    pub schedule_at: Option<DateTime<Utc>>,
    pub max_attempts: i32,
    pub metadata: Option<Value>,
    pub active: bool,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        Ok(Self { pool })
    }

    pub async fn create_job(&self, job_data: JobData) -> Result<String> {
        let id = Uuid::new_v4();
        let query = r#"
            INSERT INTO jobs (status, priority, scheduled_at, parent_job_id, max_retries, retries, payload, id, created_at, updated_at)
                    VALUES ($1, $2, $3::timestamp with time zone, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING id
        "#;

        let result = sqlx::query(query)
            .bind(job_data.status)
            .bind(job_data.priority)
            .bind(job_data.schedule_at.unwrap())
            .bind(job_data.parent_job_id)
            .bind(job_data.max_retries)
            .bind(job_data.retries)
            .bind(job_data.payload)
            .bind(&id)
            .fetch_one(&self.pool)
            .await?
            .get::<Uuid, _>("id");

        Ok(result.to_string())
    }

    pub async fn create_template(&self, job_data: JobData, job_type: JobType) -> Result<String> {
        let id = Uuid::new_v4();
        let query = r#"
            INSERT INTO templates (id, name, description, job_type, priority, max_retries, interval, cron, schedule_at, max_attempts, payload, active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
            RETURNING id
        "#;
        let result = sqlx::query(query)
            .bind(&id)
            .bind(job_data.name)
            .bind(job_data.description)
            .bind(job_type)
            .bind(job_data.priority)
            .bind(job_data.max_retries)
            .bind(job_data.interval.unwrap_or(0) as i32)
            .bind(job_data.cron)
            .bind(job_data.schedule_at.unwrap())
            .bind(job_data.max_attempts)
            .bind(job_data.payload)
            .bind(true)
            .fetch_one(&self.pool)
            .await?
            .get::<Uuid, _>("id");

        Ok(result.to_string())
    }

    pub async fn get_job(&self, id: &str) -> Result<Option<HashMap<String, String>>> {
        let uuid =
            Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid UUID format: {}", e))?;
        let row = sqlx::query("SELECT * FROM jobs WHERE id = $1")
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| row_to_hashmap(&r)))
    }

    pub async fn update_job(&self, id: &str, updates: &HashMap<&str, String>) -> Result<bool> {
        let set_clauses = updates
            .iter()
            .enumerate()
            .map(|(i, (k, _))| {
                if *k == "status" {
                    format!("{} = ${}::job_status", k, i + 2)
                } else {
                    format!("{} = ${}", k, i + 2)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "UPDATE jobs SET {}, updated_at = NOW() WHERE id = $1",
            set_clauses
        );

        let mut query_builder = sqlx::query(&query).bind(id);
        for value in updates.values() {
            query_builder = query_builder.bind(value);
        }

        let result = query_builder.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_job(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM jobs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_due_jobs(
        &self,
        limit: i64,
        _job_types: &[&str],
    ) -> Result<Vec<HashMap<String, String>>> {
        let query = r#"
            SELECT * FROM jobs 
            WHERE status::job_status = 'pending'::job_status 
            AND scheduled_at <= NOW()
            ORDER BY priority DESC, scheduled_at ASC
            LIMIT $1
        "#;

        let rows = sqlx::query(query).bind(limit).fetch_all(&self.pool).await?;

        Ok(rows.iter().map(row_to_hashmap).collect())
    }

    pub async fn get_jobs_by_status(&self, status: &str) -> Result<Vec<HashMap<String, String>>> {
        let query = r#"
            SELECT * FROM jobs 
            WHERE status::job_status = $1::job_status
            ORDER BY priority DESC, scheduled_at ASC
        "#;

        let rows = sqlx::query(query)
            .bind(status)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.iter().map(row_to_hashmap).collect())
    }

    pub async fn get_jobs_older_than(
        &self,
        cutoff_time: &str,
    ) -> Result<Vec<HashMap<String, String>>> {
        let query = r#"
            SELECT * FROM jobs 
            WHERE created_at < $1::timestamp with time zone
            ORDER BY created_at ASC
        "#;

        let rows = sqlx::query(query)
            .bind(cutoff_time)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.iter().map(row_to_hashmap).collect())
    }

    pub async fn get_jobs_by_status_and_time(
        &self,
        status: &str,
        cutoff_time: &str,
    ) -> Result<Vec<HashMap<String, String>>> {
        let query = r#"
            SELECT * FROM jobs 
            WHERE status::job_status = $1::job_status AND created_at < $2
            ORDER BY created_at ASC
        "#;

        let rows = sqlx::query(query)
            .bind(status)
            .bind(cutoff_time)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.iter().map(row_to_hashmap).collect())
    }

    pub async fn get_active_templates(&self) -> Result<Vec<Template>> {
        let query = r#"
            SELECT * FROM templates 
            WHERE active = true
            ORDER BY created_at ASC
        "#;

        let rows = sqlx::query_as::<_, Template>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }
}

fn row_to_hashmap(row: &PgRow) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for column in row.columns() {
        if let Ok(value) = row.try_get::<String, _>(column.name()) {
            map.insert(column.name().to_string(), value);
        }
    }
    map
}
