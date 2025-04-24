use rocket::figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub address: String,
    pub port: u16,
    pub workers: u16,
    pub log_level: String,
}

impl Settings {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let figment = Figment::from(rocket::Config::default())
            .merge(Toml::file("Rocket.toml"))
            .merge(Env::prefixed("ROCKET_"));

        Ok(figment.extract()?)
    }
}
