use crate::hash::{generate_salt, hash_password, verify_password};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::process::exit;

#[derive(Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    Remote,
    Local,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub root_user: Option<String>,
    pub root_password: Option<String>,
    pub api_key: Option<String>,
    pub database_type: Option<DatabaseType>,
    pub database_endpoint: Option<String>,
}

pub fn check_settings() -> (Settings, String) {
    let mut settings_exists: bool = false;
    let settings: Settings = match fs::read_to_string("settings.json") {
        Ok(contents) => {
            settings_exists = true;
            let json: Settings = serde_json::from_str(&contents).unwrap();
            json
        }
        Err(_) => Settings {
            root_user: None,
            root_password: None,
            api_key: None,
            database_type: None,
            database_endpoint: None,
        },
    };

    let mut updated_settings = settings.clone();

    let required_fields = [
        "root username",
        "root password",
        "database type",
        "database endpoint",
        "api key",
    ];

    if required_fields.iter().any(|field| match *field {
        "root username" => updated_settings.root_user.is_none(),
        "root password" => updated_settings.root_password.is_none(),
        "api key" => updated_settings.api_key.is_none(),
        "database type" => updated_settings.database_type.is_none(),
        "database endpoint" => updated_settings.database_endpoint.is_none(),
        _ => false,
    }) {
        if settings_exists {
            println!("Some fields are missing in settings.json. Let's fill them in.");
        } else {
            println!("Settings do not exist, please answer the following prompts to start.");
        }
        for field in required_fields {
            match field {
                "root username" => {
                    updated_settings.root_user = prompt_user("Set a root username: ");
                }
                "root password" => {
                    let password = prompt_user("Set a root password: ");
                    let salt = generate_salt();
                    updated_settings.root_password =
                        hash_password(password.unwrap_or_default(), salt.clone()).ok();
                }
                "api key" => {
                    let key = prompt_user("Set an API key: ");
                    let salt = generate_salt();
                    updated_settings.api_key =
                        hash_password(key.unwrap_or_default(), salt.clone()).ok();
                }
                "database type" => {
                    let db_type = prompt_user("Set database type (remote/local): ");
                    match db_type {
                        Some(type_) => {
                            updated_settings.database_type = match type_.as_str() {
                                "remote" => Some(DatabaseType::Remote),
                                "local" => Some(DatabaseType::Local),
                                _ => None,
                            };
                        }
                        None => updated_settings.database_type = None,
                    }
                }
                "database endpoint" => {
                    updated_settings.database_endpoint = prompt_user("Set the database endpoint: ");
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
                    return (updated_settings, entered_password.unwrap());
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
