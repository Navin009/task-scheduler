use config::{Config, ConfigError, Environment as ConfigEnvironment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub server: ServerSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let base_path = std::env::current_dir()
            .expect("Failed to determine the current directory")
            .parent()
            .expect("Failed to get parent directory")
            .to_path_buf();
        let configuration_directory = base_path.join("config");

        // Detect the running environment.
        // Default to `local` if unspecified.
        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");

        let environment_filename = format!("{}.yaml", environment.as_str());

        let settings = Config::builder()
            // Read the "default" configuration file
            .add_source(File::from(configuration_directory.join("base.yaml")))
            // Layer on the environment-specific values.
            .add_source(File::from(
                configuration_directory.join(environment_filename),
            ))
            // Add in settings from environment variables (with a prefix of APP and '__' as separator)
            // E.g. `APP_APPLICATION__PORT=5001` would set `Settings.application.port`
            .add_source(ConfigEnvironment::with_prefix("APP").separator("__"))
            .build()?;

        settings.try_deserialize()
    }
}

#[derive(Debug)]
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
