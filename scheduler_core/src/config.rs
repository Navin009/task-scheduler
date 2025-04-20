use serde::Deserialize;
use dotenv::dotenv;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub max_retries: u32,
    pub queue_names: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        dotenv().ok();

        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::default())?;

        cfg.try_into()
            .map_err(|e| Error::ConfigError(e.to_string()))
    }
}