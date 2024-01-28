#[macro_use]
extern crate rocket;

use tokio::task::block_in_place;

mod database;
mod hash;
mod models;
mod routes;
mod settings;
use {
    database::Database,
    routes::*,
    settings::{check_settings, Settings},
};

#[launch]
async fn rocket() -> _ {
    let mut db_settings = Settings {
        root_user: None,
        database_type: None,
        database_endpoint: None,
        api_key: None,
        root_password: None,
    };
    let mut password = Default::default();
    block_in_place(|| {
        (db_settings, password) = check_settings();
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
                signout
            ],
        )
        .manage(db)
        .manage(db_settings.api_key.unwrap())
}
