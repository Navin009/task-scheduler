use crate::JobType;
use crate::models::Template;
use anyhow::Result;
use serde_json::Value;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Column, Row};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

#[derive(Debug)]
pub struct JobData {
    pub job_type: JobType,
    pub status: String,
    pub priority: i32,
    pub scheduled_at: String,
    pub parent_job_id: Option<String>,
    pub max_retries: i32,
    pub retries: i32,
    pub payload: Value,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        Ok(Self { pool })
    }

    // Low-level job operations
    pub async fn create_job(&self, job_data: JobData) -> Result<String> {
        let query = r#"
            INSERT INTO jobs (job_type, status, priority, scheduled_at, parent_job_id, max_retries, retries, payload, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
                RETURNING id
        "#;

        let id = sqlx::query(query)
            .bind(job_data.job_type)
            .bind(job_data.status)
            .bind(job_data.priority)
            .bind(job_data.scheduled_at)
            .bind(job_data.parent_job_id)
            .bind(job_data.max_retries)
            .bind(job_data.retries)
            .bind(job_data.payload)
            .fetch_one(&self.pool)
            .await?
            .get::<String, _>("id");

        Ok(id)
    }

    pub async fn get_job(&self, id: &str) -> Result<Option<HashMap<String, String>>> {
        let row = sqlx::query("SELECT * FROM jobs WHERE id = $1")
            .bind(id)
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
        job_types: &[&str],
    ) -> Result<Vec<HashMap<String, String>>> {
        let job_types_str = job_types
            .iter()
            .map(|t| format!("'{}'", t))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            r#"
            SELECT * FROM jobs 
            WHERE status::job_status = 'pending'::job_status 
            AND scheduled_at <= NOW()
            AND job_type IN ({})
            ORDER BY priority DESC, scheduled_at ASC
            LIMIT $1
            "#,
            job_types_str
        );

        let rows = sqlx::query(&query)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

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
