#[macro_use]
extern crate rocket;

use crate::config::AppConfig;
use middleware::logging::LoggerFairing;
use rocket::{Build, Rocket};
use scheduler_core::{config::Config, init::init_database};
use security::jwt::JWTAuthenticator;

mod config;
mod error;
mod guard;
mod handlers;
mod middleware;
mod security;
mod model;

#[launch]
async fn rocket() -> Rocket<Build> {
    AppConfig::init_logger();

    let config = Config::from_env().expect("Failed to load configuration");

    let db = init_database(&config)
        .await
        .expect("Failed to initialize database connection");

    let app_config = AppConfig::new(db);

    rocket::build()
        .manage(JWTAuthenticator::new())
        .manage(app_config)
        .attach(LoggerFairing)
        .mount("/", handlers::ping_routes())
        .mount("/", handlers::jobs_routes())
}
