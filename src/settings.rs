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
    let mut settings_exists: bool = false;
    let settings: Settings = match fs::read_to_string("settings.json") {
        Ok(contents) => {
            settings_exists = true;
            let json: Settings = serde_json::from_str(&contents).unwrap();
            api_key = json.api_key.clone().unwrap_or_default();
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

    let required_fields = ["root username", "root password", "database name", "api key"];

    if required_fields.iter().any(|field| match *field {
        "root username" => updated_settings.root_user.is_none(),
        "root password" => updated_settings.root_password.is_none(),
        "database name" => updated_settings.db_name.is_none(),
        "api key" => updated_settings.api_key.is_none(),
        _ => false,
    }) {
        if settings_exists {
            println!("Some fields are missing in settings.json. Let's fill them in.");
        } else {
            println!("Settings do not exists, please answer the following prompts to start.")
        }
        for field in required_fields {
            match field {
                "root username" if updated_settings.root_user.is_none() => {
                    updated_settings.root_user = prompt_user("Set a root username: ");
                }
                "root password" if updated_settings.root_password.is_none() => {
                    let password = prompt_user("Set a root password: ");
                    let salt = generate_salt();
                    updated_settings.root_password =
                        hash_password(password.unwrap_or_default(), salt.clone()).ok();
                }
                "database name" if updated_settings.db_name.is_none() => {
                    updated_settings.db_name = prompt_user("Set a database name: ");
                }
                "api key" if updated_settings.api_key.is_none() => {
                    let key = prompt_user("Set an API key: ");
                    api_key = key.clone().unwrap_or_default();
                    let salt = generate_salt();
                    updated_settings.api_key =
                        hash_password(key.unwrap_or_default(), salt.clone()).ok();
                }
                _ => {}
            }
        }

        save_settings(&updated_settings, "settings.json");
    }
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
