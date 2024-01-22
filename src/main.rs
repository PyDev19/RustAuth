#[macro_use]
extern crate rocket;

use std::fs::File;
use std::io::{self, BufRead, Write};
use tokio::task::block_in_place;

mod routes;
use routes::*;
mod database;
use database::*;
mod hash;
use hash::*;

fn read_api_key(file: File) -> Option<String> {
    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        if let Ok(line) = line {
            return Some(line);
        }
    }
    None
}

fn write_api_key(file_name: &str, content: &str) -> io::Result<()> {
    let file_path = format!(
        "{}/{}",
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_str()
            .unwrap(),
        file_name
    );
    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[launch]
async fn rocket() -> _ {
    let mut api_key = String::from("");

    block_in_place(|| {
        if let Ok(file) = File::open(format!(
            "{}/{}",
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_str()
                .unwrap(),
            "key.txt"
        )) {
            if let Some(first_line) = read_api_key(file) {
                api_key = first_line;
            } else {
                print!("key.txt is empty. Please enter the key: ");
                let _flush = io::stdout().flush();
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                let salt = generate_salt();
                let api_key = hash_password(input.trim().to_string(), salt.clone()).ok();
                if let Err(err) = write_api_key("key.txt", api_key.unwrap().as_str()) {
                    eprintln!("Error writing to api.txt: {}", err);
                }
            }
        } else {
            print!("key.txt not found. Please enter the key: ");
            let _flush = io::stdout().flush();
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");
            let salt = generate_salt();
            let api_key = hash_password(input.trim().to_string(), salt.clone()).ok();
            if let Err(err) = write_api_key("key.txt", api_key.unwrap().as_str()) {
                eprintln!("Error writing to api.txt: {}", err);
            }
        }
    });

    let db = Database::new().await.expect("error connecting to database");
    rocket::build()
        .mount(
            "/",
            routes![root, signup, get_user, delete_user, email_login],
        )
        .manage(db)
        .manage(api_key)
}
