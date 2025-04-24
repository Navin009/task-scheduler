use anyhow::Result;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Column, Row};
use std::collections::HashMap;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        Ok(Self { pool })
    }

    // Low-level job operations
    pub async fn create_job(&self, job: &HashMap<&str, String>) -> Result<String> {
        let columns = job.keys().map(|s| *s).collect::<Vec<_>>().join(", ");
        let values = job.values().collect::<Vec<_>>();
        let placeholders = (1..=values.len())
            .map(|i| format!("${}", i))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "INSERT INTO jobs ({}) VALUES ({}) RETURNING id",
            columns, placeholders
        );

        let mut query_builder = sqlx::query(&query);
        for value in values {
            query_builder = query_builder.bind(value);
        }

        let id = query_builder
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
            .map(|(i, (k, _))| format!("{} = ${}", k, i + 2))
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
            WHERE status = 'pending' 
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
