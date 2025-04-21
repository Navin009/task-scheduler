#[macro_use]
extern crate rocket;

mod config;
mod error;
mod routes;

use config::Settings;
use rocket::fairing::AdHoc;

#[launch]
async fn rocket() -> _ {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let _settings = Settings::new().expect("Failed to load configuration");

    rocket::build()
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
