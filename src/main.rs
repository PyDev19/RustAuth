#[macro_use]
extern crate rocket;

mod routes;
use routes::*;
mod database;
use database::*;
mod hash;

#[launch]
async fn rocket() -> _ {
    let db = Database::new().await.expect("error connecting to database");
    rocket::build().mount("/", routes![root, signup, get_user, delete_user, email_login]).manage(db)
}
