use sqlx::{Pool, Postgres};
use crate::{Job, JobStatus, Error};

pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = Pool::<Postgres>::connect(database_url).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Database { pool })
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

    pub async fn get_pending_jobs(&self, batch_size: i64) -> Result<Vec<Job>, Error> {
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
