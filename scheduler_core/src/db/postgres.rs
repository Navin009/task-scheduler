use crate::error::{Result, SchedulerError};
use crate::task::{Task, TaskStatus};
use chrono::{DateTime, Utc};
use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;

// --- Database Schema (Example - Use migrations in a real project) ---
/*
-- Required PostgreSQL extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enum type for task status
DO $$ BEGIN
    CREATE TYPE task_status AS ENUM ('pending', 'running', 'completed', 'failed', 'retry');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    schedule_str VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    status task_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    next_run_time TIMESTAMPTZ,
    last_run_time TIMESTAMPTZ,
    last_error TEXT,
    max_retries INT NOT NULL DEFAULT 3,
    current_retries INT NOT NULL DEFAULT 0
);

-- Index for fetching due tasks efficiently
CREATE INDEX IF NOT EXISTS idx_tasks_next_run_time_status ON tasks (next_run_time, status);
-- Index for finding tasks by status
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks (status);
*/

/// Establishes a connection pool to the PostgreSQL database.
pub async fn connect(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10) // Configure pool size as needed
        .connect(database_url)
        .await
        .map_err(|e| SchedulerError::Initialization(format!("Failed to connect to Postgres: {}", e)))
}

/// Creates a new task in the database.
pub async fn create_task(pool: &PgPool, task: &Task) -> Result<Uuid> {
    let task_id = sqlx::query_scalar!(
        r#"
        INSERT INTO tasks (id, name, schedule_str, payload, status, created_at, updated_at, next_run_time, max_retries, current_retries)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        task.id,
        task.name,
        task.schedule, // Renamed column in query
        &task.payload, // Pass payload as &Value
        task.status as TaskStatus, // Cast enum
        task.created_at,
        task.updated_at,
        task.next_run_time,
        task.max_retries,
        task.current_retries
    )
    .fetch_one(pool)
    .await?;

    Ok(task_id)
}

/// Retrieves a task by its ID.
pub async fn get_task(pool: &PgPool, task_id: Uuid) -> Result<Option<Task>> {
    let task = sqlx::query_as!(
        Task,
        r#"
        SELECT
            id, name, schedule_str as schedule, payload, status AS "status!: TaskStatus",
            created_at, updated_at, next_run_time, last_run_time, last_error,
            max_retries, current_retries
        FROM tasks
        WHERE id = $1
        "#,
        task_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

/// Updates the status and potentially the last error of a task.
pub async fn update_task_status(
    pool: &PgPool,
    task_id: Uuid,
    status: TaskStatus,
    last_error: Option<String>,
) -> Result<()> {
    let now = Utc::now();
    sqlx::query!(
        r#"
        UPDATE tasks
        SET status = $1, last_error = $2, updated_at = $3
        WHERE id = $4
        "#,
        status as TaskStatus,
        last_error,
        now,
        task_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Updates the next run time and potentially resets retry count for a task.
pub async fn update_task_next_run(
    pool: &PgPool,
    task_id: Uuid,
    next_run_time: Option<DateTime<Utc>>,
    reset_retries: bool,
) -> Result<()> {
    let now = Utc::now();
    let current_retries = if reset_retries { Some(0) } else { None }; // Option<i32>

    sqlx::query!(
        r#"
        UPDATE tasks
        SET
            next_run_time = $1,
            updated_at = $2,
            current_retries = COALESCE($3, current_retries) -- Only update if $3 is not NULL
        WHERE id = $4
        "#,
        next_run_time,
        now,
        current_retries, // Pass Option<i32>
        task_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Updates fields relevant when a task starts running.
pub async fn mark_task_as_running(pool: &PgPool, task_id: Uuid) -> Result<()> {
    let now = Utc::now();
    sqlx::query!(
        r#"
        UPDATE tasks
        SET status = $1, last_run_time = $2, updated_at = $2, current_retries = current_retries + 1
        WHERE id = $3 AND status != $1 -- Avoid re-marking if already running
        "#,
        TaskStatus::Running as TaskStatus,
        now,
        task_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Fetches tasks that are due to run (next_run_time <= now and status is Pending or Retry).
pub async fn get_due_tasks(pool: &PgPool, now: DateTime<Utc>, limit: i64) -> Result<Vec<Task>> {
    let tasks = sqlx::query_as!(
        Task,
        r#"
        SELECT
            id, name, schedule_str as schedule, payload, status AS "status!: TaskStatus",
            created_at, updated_at, next_run_time, last_run_time, last_error,
            max_retries, current_retries
        FROM tasks
        WHERE
            next_run_time <= $1
            AND status IN ($2, $3) -- Pending or Retry
            AND current_retries < max_retries
        ORDER BY next_run_time ASC
        LIMIT $4
        "#,
        now,
        TaskStatus::Pending as TaskStatus,
        TaskStatus::Retry as TaskStatus,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

/// Deletes a task by its ID.
pub async fn delete_task(pool: &PgPool, task_id: Uuid) -> Result<bool> {
    let result = sqlx::query!(
        r#"
        DELETE FROM tasks
        WHERE id = $1
        "#,
        task_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
