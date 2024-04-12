use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub username: String,
    pub password: String,
    pub logged_in: bool,
    pub recovery_code: Option<i32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignUp {
    pub email: String,
    pub username: String,
    pub password: String,
    pub api_key: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailLogin {
    pub email: String,
    pub password: String,
    pub api_key: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginSuccess {
    pub email: String,
    pub username: String    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernameLogin {
    pub username: String,
    pub password: String,
    pub api_key: String
}
