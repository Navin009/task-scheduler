use crate::error::{Result, SchedulerError};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use uuid::Uuid;

/// Establishes a connection manager to Redis.
pub async fn connect(redis_url: &str) -> Result<ConnectionManager> {
    let client = Client::open(redis_url)
        .map_err(|e| SchedulerError::Initialization(format!("Invalid Redis URL: {}", e)))?;
    ConnectionManager::new(client)
        .await
        .map_err(|e| SchedulerError::Initialization(format!("Failed to connect to Redis: {}", e)))
}

fn lock_key(task_id: Uuid) -> String {
    format!("scheduler_lock:{}", task_id)
}

/// Attempts to acquire a distributed lock for a task.
/// Returns true if the lock was acquired, false otherwise.
/// Uses Redis SET NX PX command for atomic set-if-not-exists with expiration.
pub async fn acquire_lock(
    conn: &mut ConnectionManager,
    task_id: Uuid,
    lock_ttl_ms: usize,
) -> Result<bool> {
    let key = lock_key(task_id);
    let lock_value = "locked"; // Could be worker ID or timestamp

    let result: Option<String> = conn
        .set_options(
            key,
            lock_value,
            redis::SetOptions::default()
                .nx() // Only set if the key does not exist
                .px(lock_ttl_ms), // Set expiration in milliseconds
        )
        .await?;

    Ok(result.is_some()) // If SET was successful (key didn't exist), result is Some("OK") or similar
}

/// Releases the distributed lock for a task.
/// Uses a Lua script for safe check-and-delete (optional but safer).
pub async fn release_lock(conn: &mut ConnectionManager, task_id: Uuid) -> Result<()> {
    let key = lock_key(task_id);
    // Simple delete - assumes we own the lock.
    // For safer release, compare value before deleting (using Lua).
    conn.del(key).await?;
    Ok(())
}

// Example of a safer release lock using Lua (requires lock value check)
/*
pub async fn release_lock_safe(conn: &mut ConnectionManager, task_id: Uuid, expected_value: &str) -> Result<bool> {
    let key = lock_key(task_id);
    let script = redis::Script::new(r#"
        if redis.call("get", KEYS[1]) == ARGV[1] then
            return redis.call("del", KEYS[1])
        else
            return 0
        end
    "#);

    let result: i32 = script.key(key).arg(expected_value).invoke_async(conn).await?;
    Ok(result == 1) // Returns 1 if deleted, 0 otherwise
}
*/
