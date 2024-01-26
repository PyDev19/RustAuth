#[macro_use]
extern crate rocket;

use tokio::task::block_in_place;

mod database;
mod hash;
mod models;
mod routes;
mod settings;
use routes::*;
use database::Database;
use settings::check_settings;

#[launch]
async fn rocket() -> _ {
    let mut api_key = String::from("");
    let mut root_pass = String::from("");
    let mut root_user = String::from("");
    let mut db_name = String::from("");

    block_in_place(|| {
        let settings_ = check_settings();
        (root_user, root_pass, db_name, api_key) = settings_;
    });

    let db = Database::new(db_name, root_user, root_pass)
        .await
        .expect("Error connecting to database");
    rocket::build()
        .mount(
            "/",
            routes![
                root,
                signup,
                get_user,
                delete_user,
                email_login,
                username_login,
                signout
            ],
        )
        .manage(db)
        .manage(api_key)
}
