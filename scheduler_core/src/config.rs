use serde::Deserialize;
use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub max_retries: u32,
    pub queue_names: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        let cfg = config::Config::builder()
            .add_source(config::Environment::default())
            .build()
            .map_err(|e| Error::ConfigError(e.to_string()))?;

        cfg.try_deserialize()
            .map_err(|e| Error::ConfigError(e.to_string()))
    }
}