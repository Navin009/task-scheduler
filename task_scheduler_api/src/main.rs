#[macro_use]
extern crate rocket;

mod config;
mod routes;
mod error;

use config::Settings;
use rocket::fairing::AdHoc;
use rocket::figment::Figment;
use rocket::Config as RocketConfig;
use std::env;

#[launch]
async fn rocket() -> _ {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let settings = Settings::new().expect("Failed to load configuration");

    // Configure Rocket
    let figment = Figment::from(RocketConfig::default())
        .merge(("port", settings.server.port))
        .merge(("address", settings.server.host.clone()));

    rocket::custom(figment)
        .attach(AdHoc::config::<Settings>())
        .mount("/api", routes![
            routes::health::health_check,
            routes::jobs::create_job,
            routes::jobs::get_job,
            routes::jobs::list_jobs,
            routes::jobs::update_job,
            routes::jobs::delete_job,
            routes::templates::create_template,
            routes::templates::get_template,
            routes::templates::list_templates,
            routes::templates::update_template,
            routes::templates::delete_template,
        ])
        .register("/", catchers![
            error::not_found,
            error::unprocessable_entity,
            error::internal_server_error
        ])
}
