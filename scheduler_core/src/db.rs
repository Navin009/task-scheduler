use crate::models::ScheduleType;
use crate::{Error, Job, JobStatus};
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Database { pool }
    }

    pub async fn execute_query(&self, query: &str) -> Result<Vec<sqlx::postgres::PgRow>, Error> {
        let rows = sqlx::query(query).fetch_all(&self.pool).await?;
        Ok(rows)
    }

    pub async fn create_job(&self, job: &Job) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO jobs 
            (id, schedule_type, schedule, payload, status, retries, max_retries)
            VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(&job.id)
        .bind(&job.schedule_type)
        .bind(&job.schedule)
        .bind(&job.payload)
        .bind(&job.status)
        .bind(job.retries)
        .bind(job.max_retries)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_pending_jobs(&self, batch_size: i64) -> Result<Vec<Job>, Error> {
        let jobs = sqlx::query_as::<_, Job>(
            r#"SELECT * FROM jobs 
            WHERE status = $1
            ORDER BY created_at ASC
            LIMIT $2"#,
        )
        .bind(JobStatus::Pending)
        .bind(batch_size)
        .fetch_all(&self.pool)
        .await?;
        Ok(jobs)
    }

    pub async fn get_job(&self, id: &str) -> Result<Option<Job>, Error> {
        let job = sqlx::query_as::<_, Job>(r#"SELECT * FROM jobs WHERE id = $1"#)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(job)
    }

    pub async fn update_job(
        &self,
        id: &str,
        schedule_type: Option<ScheduleType>,
        schedule: Option<&str>,
        payload: Option<&serde_json::Value>,
        max_retries: Option<i32>,
    ) -> Result<Option<Job>, Error> {
        let job = sqlx::query_as::<_, Job>(
            r#"UPDATE jobs 
            SET schedule_type = COALESCE($2, schedule_type),
                schedule = COALESCE($3, schedule),
                payload = COALESCE($4, payload),
                max_retries = COALESCE($5, max_retries),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING *"#,
        )
        .bind(id)
        .bind(schedule_type)
        .bind(schedule)
        .bind(payload)
        .bind(max_retries)
        .fetch_optional(&self.pool)
        .await?;
        Ok(job)
    }

    pub async fn delete_job(&self, id: &str) -> Result<(), Error> {
        sqlx::query(r#"DELETE FROM jobs WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
