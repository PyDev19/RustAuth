use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub username: String,
    pub password: String,
    pub logged_in: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignUp {
    pub email: String,
    pub username: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailLoginIn {
    pub email: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailLoginInSuccess {
    pub email: String,
    pub username: String    
}
