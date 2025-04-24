use sqlx::{Pool, Postgres};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub struct MigrationManager {
    pool: Pool<Postgres>,
    migrations_dir: String,
}

impl MigrationManager {
    pub fn new(pool: Pool<Postgres>, migrations_dir: impl Into<String>) -> Self {
        Self {
            pool,
            migrations_dir: migrations_dir.into(),
        }
    }

    pub async fn run_migrations(&self) -> Result<(), MigrationError> {
        log::info!("Running migrations from directory: {}", self.migrations_dir);
        
        // Verify migrations directory exists
        if !Path::new(&self.migrations_dir).exists() {
            return Err(MigrationError::MigrationError(format!(
                "Migrations directory not found: {}",
                self.migrations_dir
            )));
        }

        // Run migrations
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .map_err(|e| MigrationError::MigrationError(e.to_string()))?;

        log::info!("Migrations completed successfully");
        Ok(())
    }

    pub async fn check_migrations(&self) -> Result<bool, MigrationError> {
        // Check if migrations table exists
        let result = sqlx::query(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = '_sqlx_migrations'
            )"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| MigrationError::MigrationError(e.to_string()))?;

        let exists: bool = result.get(0);
        Ok(exists)
    }

    pub async fn get_applied_migrations(&self) -> Result<Vec<String>, MigrationError> {
        let migrations = sqlx::query(
            "SELECT version FROM _sqlx_migrations ORDER BY version"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| MigrationError::MigrationError(e.to_string()))?;

        Ok(migrations
            .iter()
            .map(|row| row.get::<String, _>("version"))
            .collect())
    }
} 