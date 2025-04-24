#[macro_use]
extern crate rocket;

use crate::config::AppConfig;
use middleware::logging::LoggerFairing;
use rocket::{Build, Rocket};
use security::jwt::JWTAuthenticator;

mod config;
mod error;
mod guard;
mod handlers;
mod middleware;
mod security;

#[launch]
async fn rocket() -> Rocket<Build> {
    AppConfig::init_logger();

    let config = AppConfig::init_app_config()
        .await
        .expect("Failed to initialize app config");

    let postgres = AppConfig::init_db(&config)
        .await
        .expect("Failed to initialize database connection");

    let app_config = AppConfig::new(postgres);

    rocket::build()
        .manage(JWTAuthenticator::new())
        .manage(app_config)
        .attach(LoggerFairing)
        .mount("/", handlers::ping_routes())
        .mount("/", handlers::jobs_routes())
        .mount("/", handlers::templates_routes())
}
