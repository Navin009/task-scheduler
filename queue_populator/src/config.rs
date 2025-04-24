use scheduler_core::config::Config;

#[derive(Debug)]
pub struct QueuePopulatorConfig {
    pub database_url: String,
    pub cache_url: String,
    pub max_connections: u32,
    pub poll_interval_seconds: u64,
}

impl QueuePopulatorConfig {
    pub fn from_core_config(config: &Config) -> Self {
        Self {
            database_url: config.database.url.clone(),
            cache_url: config.cache.url.clone(),
            max_connections: config.cache.max_connections,
            poll_interval_seconds: 1, // Default poll interval
        }
    }
}
