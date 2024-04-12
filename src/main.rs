#![warn(clippy::pedantic)]

#[macro_use]
extern crate rocket;

use rocket::{tokio::task::block_in_place, Build, Rocket};

mod database;
mod hash;
mod models;
mod routes;
mod settings;
use {
    database::Database,
    routes::{
        account_recovery, delete_user, email_login, get_user, root, signout, signup, username_login,
    },
    settings::{check_json, Settings},
};

#[launch]
async fn rocket() -> Rocket<Build> {
    let mut db_settings = Settings {
        root_user: None,
        database_type: None,
        database_endpoint: None,
        api_key: None,
        root_password: None,
    };
    let mut password = String::default();
    block_in_place(|| {
        (db_settings, password) = check_json();
    });

    let db = Database::new(db_settings.clone(), password)
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
                signout,
                account_recovery
            ],
        )
        .manage(db)
        .manage(db_settings.api_key.unwrap())
}
