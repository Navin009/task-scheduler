#[macro_use]
extern crate rocket;

mod controller;
mod model;
mod service;

use controller::job_controller::{retrieve_job, schedule_job};
use service::job_service::JobService;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(JobService::new())
        .mount("/api", routes![schedule_job, retrieve_job])
}
