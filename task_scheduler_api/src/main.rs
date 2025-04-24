#[macro_use]
extern crate rocket;

mod config;
mod error;
mod routes;

use config::Settings;
use rocket::fairing::AdHoc;
use rocket_db_pools::{Database, sqlx};
use std::net::IpAddr;

#[derive(Database)]
#[database("task_scheduler")]
pub struct Db(rocket_db_pools::sqlx::Pool<sqlx::Postgres>);

#[launch]
async fn rocket() -> _ {
    // Load configuration
    let settings = Settings::new().expect("Failed to load configuration");

    // Initialize logger with configured settings
    let log_level = settings.log_level.parse().unwrap_or(log::LevelFilter::Info);
    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp_secs()
        .init();

    rocket::build()
        .configure(
            rocket::Config::figment()
                .merge(("address", settings.address.parse::<IpAddr>().unwrap()))
                .merge(("port", settings.port))
                .merge(("workers", settings.workers)),
        )
        .attach(Db::init())
        .attach(AdHoc::config::<Settings>())
        .mount(
            "/api",
            routes![
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
            ],
        )
        .register(
            "/",
            catchers![
                error::not_found,
                error::unprocessable_entity,
                error::internal_server_error
            ],
        )
}
