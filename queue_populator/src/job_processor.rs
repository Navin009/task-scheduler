use scheduler_core::{cache::Cache, db::Database, task::TaskManager, JobStatus};
use tracing::error;

use crate::error::{QueuePopulatorError, Result};

pub struct JobProcessor {
    cache: Cache,
    task_manager: TaskManager,
}

impl JobProcessor {
    pub async fn new(cache: Cache, database_url: &str) -> Result<Self> {
        let db = Database::new(database_url)
            .await
            .map_err(QueuePopulatorError::from)?;
        let task_manager = TaskManager::new(db);
        Ok(Self {
            cache,
            task_manager,
        })
    }

    pub async fn process_jobs(&self) -> Result<()> {
        let due_jobs = self.fetch_due_jobs().await?;

        for job in due_jobs {
            if let Err(e) = self.push_job_to_queue(&job).await {
                error!("Failed to push job {} to queue: {}", job.id, e);
                continue;
            }

            if let Err(e) = self.update_job_status(&job).await {
                error!("Failed to update job {} status: {}", job.id, e);
            }
        }

        Ok(())
    }

    async fn fetch_due_jobs(&self) -> Result<Vec<scheduler_core::task::Job>> {
        self.task_manager
            .get_due_jobs(100)
            .await
            .map_err(QueuePopulatorError::from)
    }

    async fn push_job_to_queue(&self, job: &scheduler_core::task::Job) -> Result<()> {
        let job_json = serde_json::to_string(job)?;
        let queue_name = format!("jobs:{}", job.priority);

        self.cache
            .push_to_queue(&queue_name, &job_json)
            .await
            .map_err(QueuePopulatorError::from)
    }

    async fn update_job_status(&self, job: &scheduler_core::task::Job) -> Result<()> {
        self.task_manager
            .update_job_status(&job.id, JobStatus::Pending)
            .await?;

        if let Some(queue_name) = self.get_queue_name(&job) {
            self.cache
                .push_to_priority_queue(&queue_name, &job.id, job.priority)
                .await?;
        }

        Ok(())
    }

    fn get_queue_name(&self, job: &scheduler_core::task::Job) -> Option<String> {
        None
    }
}
