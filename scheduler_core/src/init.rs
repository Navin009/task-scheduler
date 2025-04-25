use crate::{
    cache::{Cache, CacheConfig},
    config::Config,
    db::Database,
};
use anyhow::Result;

/// Initialize the database connection using the configuration
pub async fn init_database(config: &Config) -> Result<Database> {
    Database::new(&config.database_url).await
}

/// Initialize the cache connection using the configuration
pub async fn init_cache(config: &Config) -> Result<Cache> {
    let cache_config = CacheConfig {
        url: config.redis_url.clone(),
        max_connections: 10, // Default max connections
    };
    Cache::new(cache_config).await
}
