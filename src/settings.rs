use crate::hash::{generate_salt, hash_password, verify_password};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::process::exit;

#[derive(Clone, Serialize, Deserialize)]
struct Settings {
    root_user: Option<String>,
    root_password: Option<String>,
    db_name: Option<String>,
    api_key: Option<String>,
}

pub fn check_settings() -> (String, String, String, String) {
    let mut api_key: String = String::from("");
    let settings: Settings = match fs::read_to_string("settings.json") {
        Ok(contents) => {
            let json: Settings = serde_json::from_str(&contents).unwrap();
            api_key = json.api_key.clone().unwrap();
            json
        }
        Err(_) => Settings {
            root_user: None,
            root_password: None,
            db_name: None,
            api_key: None,
        },
    };

    let mut updated_settings = settings.clone();

    if settings.root_user.is_none() {
        println!("Root username not found in settings.json");
        updated_settings.root_user = prompt_user("What root username do you want: ");
        print!("\x1B[2J\x1B[1;1H");
    }

    if settings.root_password.is_none() {
        println!("Root password not found in settings.json");
        let password = prompt_user("What root password do you want: ");
        let salt = generate_salt();
        updated_settings.root_password = hash_password(password.unwrap(), salt.clone()).ok();
        print!("\x1B[2J\x1B[1;1H");
    }

    if settings.db_name.is_none() {
        println!("Database file name not found in settings.json");
        updated_settings.db_name = prompt_user("What database file name do you want: ");
        print!("\x1B[2J\x1B[1;1H");
    }

    if settings.api_key.is_none() {
        println!("API key not found in settings.json");
        let key = prompt_user("What API key will you use: ");
        api_key = key.clone().unwrap();
        let salt = generate_salt();
        updated_settings.api_key = hash_password(key.unwrap(), salt.clone()).ok();
        print!("\x1B[2J\x1B[1;1H");
    }

    save_settings(&updated_settings, "settings.json");

    print!("\x1B[2J\x1B[1;1H");
    
    for attempt in (0..3).rev() {
        let entered_password = prompt_user("Enter root password for verification: ");
        match verify_password(
            entered_password.clone().unwrap(),
            updated_settings.root_password.clone().unwrap(),
        ) {
            Ok(result) => {
                if result {
                    println!("Password verified. Starting Rocket Server...");
                    return (
                        updated_settings.root_user.clone().unwrap(),
                        entered_password.clone().unwrap(),
                        updated_settings.db_name.clone().unwrap(),
                        api_key,
                    );
                } else {
                    println!(
                        "Password verification failed. {} attempts remaining.",
                        attempt
                    );
                }
            }
            Err(_err) => exit(-1),
        }
    }

    exit(-1);
}

fn prompt_user(prompt: &str) -> Option<String> {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        None
    } else {
        Some(trimmed_input.to_string())
    }
}

fn save_settings(settings: &Settings, path: &str) {
    let serialized = serde_json::to_string_pretty(settings).unwrap();
    fs::write(path, serialized).expect("Failed to write settings to file.");
}
