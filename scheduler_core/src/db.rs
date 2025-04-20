use crate::{Config, Error};
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.database_url)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn create_job(&self, job: &Job) -> Result<(), Error> {
        sqlx::query!(
            r#"INSERT INTO jobs 
            (id, schedule_type, schedule, payload, status, retries, max_retries)
            VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            job.id,
            job.schedule_type as _,
            job.schedule,
            job.payload,
            job.status as _,
            job.retries,
            job.max_retries
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_due_jobs(&self, batch_size: i64) -> Result<Vec<Job>, Error> {
        let jobs = sqlx::query_as!(
            Job,
            r#"SELECT * FROM jobs 
            WHERE status = $1
            ORDER BY created_at ASC
            LIMIT $2"#,
            JobStatus::Pending as _,
            batch_size
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }
}
