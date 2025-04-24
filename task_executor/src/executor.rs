use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info};

use scheduler_core::{
    cache::Cache,
    db::Database,
    task::{Job, JobStatus},
};

use crate::{error::Error, process::ProcessManager, state::ExecutionState};

#[derive(Clone)]
pub struct TaskExecutor {
    db: Database,
    cache: Cache,
    process_manager: ProcessManager,
    concurrency_limit: usize,
    semaphore: Semaphore,
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
            db,
            cache,
            process_manager,
            concurrency_limit,
            semaphore: Semaphore::new(concurrency_limit),
        })
    }

    pub async fn start(&self) -> Result<(), Error> {
        info!(
            "Starting task executor with concurrency limit: {}",
            self.concurrency_limit
        );

        loop {
            // Wait for a permit before processing next job
            let _permit =
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
                    });
                }
                Ok(None) => {
                    // No jobs available, wait a bit before trying again
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Error getting job from queue: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn get_next_job(&self) -> Result<Option<Job>, Error> {
        if let Some(job_id) = self.cache.pop_from_queue("jobs").await? {
            if let Some(job) = self.db.get_job(&job_id).await? {
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
        updates.insert("status", JobStatus::Running.to_string());
        self.db.update_job(&state.job.id, &updates).await?;

        // Parse job payload
        let payload: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&state.job.payload)?)
                .map_err(|e| Error::Process(format!("Invalid job payload: {}", e)))?;

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
                updates.insert("status", JobStatus::Completed.to_string());
                self.db.update_job(&state.job.id, &updates).await?;
            }
            Err(e) => {
                let error_str = e.to_string();
                state.mark_failed(error_str.clone())?;
                let mut updates = std::collections::HashMap::new();
                updates.insert("status", JobStatus::Failed.to_string());
                self.db.update_job(&state.job.id, &updates).await?;

                // Check if we should retry
                if state.job.attempts < state.job.max_attempts {
                    if let Ok(()) = state.mark_retrying() {
                        let mut updates = std::collections::HashMap::new();
                        updates.insert("status", JobStatus::Pending.to_string());
                        updates.insert("attempts", (state.job.attempts + 1).to_string());
                        self.db.update_job(&state.job.id, &updates).await?;
                        self.cache.push_to_queue("jobs", &state.job.id).await?;
                    }
                }
            }
        }

        Ok(())
    }
}
