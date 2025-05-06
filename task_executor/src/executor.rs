use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info};

use scheduler_core::{
    cache::Cache,
    db::Database,
    models::{Job, JobStatus},
};

use crate::{error::Error, process::ProcessManager, state::ExecutionState};

#[derive(Clone)]
pub struct TaskExecutor {
    db: Arc<Database>,
    cache: Arc<Cache>,
    process_manager: Arc<ProcessManager>,
    concurrency_limit: usize,
    semaphore: Arc<Semaphore>,
}

impl TaskExecutor {
    pub async fn new(
        db: Database,
        cache: Cache,
        timeout: Duration,
        max_memory_mb: u64,
        max_cpu_percent: u32,
        concurrency_limit: usize,
    ) -> Result<Self, Error> {
        let process_manager = ProcessManager::new(timeout, max_memory_mb, max_cpu_percent);
        process_manager.validate_resources()?;

        Ok(Self {
            db: Arc::new(db),
            cache: Arc::new(cache),
            process_manager: Arc::new(process_manager),
            concurrency_limit,
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
        })
    }

    pub async fn start(&self) -> Result<(), Error> {
        info!(
            "Starting task executor with concurrency limit: {}",
            self.concurrency_limit
        );

        loop {
            // Wait for a permit before processing next job
            let permit =
                self.semaphore.acquire().await.map_err(|e| {
                    Error::ResourceLimit(format!("Failed to acquire semaphore: {}", e))
                })?;

            // Try to get next job from queue
            match self.get_next_job().await {
                Ok(Some(job)) => {
                    let executor = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = executor.execute_job(job).await {
                            error!("Failed to execute job: {}", e);
                        }
                        // Permit is automatically released when the task completes
                    });
                }
                Ok(None) => {
                    // No jobs available, wait a bit before trying again
                    drop(permit); // Explicitly release the permit
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Error getting job from queue: {}", e);
                    drop(permit); // Explicitly release the permit
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn get_next_job(&self) -> Result<Option<Job>, Error> {
        if let Some(job_id) = self.cache.pop_from_queue("jobs").await? {
            if let Some(job_data) = self.db.get_job(&job_id).await? {
                // Convert HashMap to Job
                let job = Job {
                    id: job_data.get("id").unwrap().clone(),
                    schedule: chrono::DateTime::parse_from_rfc3339(
                        job_data.get("schedule").unwrap(),
                    )
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                    payload: serde_json::from_str(job_data.get("payload").unwrap())?,
                    status: match job_data.get("status").unwrap().as_str() {
                        "pending" => JobStatus::Pending,
                        "running" => JobStatus::Running,
                        "completed" => JobStatus::Completed,
                        "failed" => JobStatus::Failed,
                        "retrying" => JobStatus::Retrying,
                        _ => return Err(Error::Process("Invalid job status".into())),
                    },
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        job_data.get("created_at").unwrap(),
                    )
                    .map_err(|e| Error::Process(format!("Invalid created_at: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        job_data.get("updated_at").unwrap(),
                    )
                    .map_err(|e| Error::Process(format!("Invalid updated_at: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    retries: job_data
                        .get("retries")
                        .unwrap()
                        .parse()
                        .map_err(|e| Error::Process(format!("Invalid retries: {}", e)))?,
                    max_retries: job_data
                        .get("max_retries")
                        .unwrap()
                        .parse()
                        .map_err(|e| Error::Process(format!("Invalid max_retries: {}", e)))?,
                };
                return Ok(Some(job));
            }
        }
        Ok(None)
    }

    async fn execute_job(&self, job: Job) -> Result<(), Error> {
        let mut state = ExecutionState::new(job);

        // Mark job as running
        state.mark_running()?;
        let mut updates = std::collections::HashMap::new();
        updates.insert("status", format!("{:?}", JobStatus::Running));
        self.db.update_job(&state.job.id, &updates).await?;

        // Parse job payload
        let payload: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&state.job.payload)?)?;

        // Extract command and arguments
        let command = payload["command"]
            .as_str()
            .ok_or_else(|| Error::Process("Missing command in payload".into()))?;
        let args: Vec<String> = payload["args"]
            .as_array()
            .ok_or_else(|| Error::Process("Missing args in payload".into()))?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        // Execute command
        match self
            .process_manager
            .execute_command(command, &args, &[])
            .await
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout).into();
                state.mark_completed(output_str)?;
                let mut updates = std::collections::HashMap::new();
                updates.insert("status", format!("{:?}", JobStatus::Completed));
                self.db.update_job(&state.job.id, &updates).await?;
            }
            Err(e) => {
                let error_str = e.to_string();
                state.mark_failed(error_str.clone())?;
                let mut updates = std::collections::HashMap::new();
                updates.insert("status", format!("{:?}", JobStatus::Failed));
                self.db.update_job(&state.job.id, &updates).await?;

                // Check if we should retry
                if state.job.retries < state.job.max_retries {
                    let mut updates = std::collections::HashMap::new();
                    updates.insert("status", format!("{:?}", JobStatus::Pending));
                    updates.insert("retries", (state.job.retries + 1).to_string());
                    self.db.update_job(&state.job.id, &updates).await?;
                    self.cache.push_to_queue("jobs", &state.job.id).await?;
                }
            }
        }

        Ok(())
    }
}
