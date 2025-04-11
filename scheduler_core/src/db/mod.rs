// src/db/mod.rs
pub mod postgres;

// Re-export key functions/types if desired
pub use postgres::{
    connect as connect_postgres, create_task, delete_task, get_due_tasks, get_task,
    mark_task_as_running, update_task_next_run, update_task_status,
};
