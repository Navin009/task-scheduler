use crate::Result;
use crate::cache;
use crate::db;
use crate::task::Task;

/// A service to handle job-related operations.
pub struct JobService {
    pub db_pool: db::PgPool,
    pub cache_conn: cache::ConnectionManager,
}

impl JobService {
    /// Create a new JobService instance.
    pub fn new(db_pool: db::PgPool, cache_conn: cache::ConnectionManager) -> Self {
        JobService {
            db_pool,
            cache_conn,
        }
    }

    /// Fetch due tasks from the database.
    pub async fn fetch_due_tasks(&self, limit: usize) -> Result<Vec<Task>> {
        db::get_due_tasks(&self.db_pool, chrono::Utc::now(), limit).await
    }

    /// Acquire a lock for a task in the cache.
    pub async fn acquire_task_lock(&mut self, task_id: i32, ttl_ms: usize) -> Result<bool> {
        cache::acquire_lock(&mut self.cache_conn, task_id, ttl_ms).await
    }

    /// Release a lock for a task in the cache.
    pub async fn release_task_lock(&mut self, task_id: i32) -> Result<()> {
        cache::release_lock(&mut self.cache_conn, task_id).await
    }

    /// Update the status of a task in the database.
    pub async fn update_task_status(
        &self,
        task_id: i32,
        status: TaskStatus,
        error_msg: Option<String>,
    ) -> Result<()> {
        db::update_task_status(&self.db_pool, task_id, status, error_msg).await
    }
}
