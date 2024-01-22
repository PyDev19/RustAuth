use crate::database::Database;
use crate::hash::*;
use crate::models::*;
use rocket::serde::json::Json;
use rocket::State;
use surrealdb::{error::Db::Thrown, Error, Error::Db};

#[post("/signup", data = "<user>")]
pub async fn signup(
    user: Json<SignUp>,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<Json<User>, Json<Error>> {
    let verify_key = verify_password(user.api_key.clone(), api_key.to_string())
        .ok()
        .unwrap();
    if verify_key {
        let created_user = db.signup(user.into_inner()).await;

        match created_user {
            Ok(result) => match result {
                Some(user) => Ok(Json(user)),
                None => Err(Json(Db(Thrown("An error occured".to_string())))),
            },
            Err(err) => Err(Json(err)),
        }
    } else {
        Err(Json(Db(Thrown("Api key is invalid".to_string()))))
    }
}

#[get("/get_user/<username>?<key>")]
pub async fn get_user(
    username: String,
    key: String,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<Json<User>, Json<Error>> {
    let verify_key = verify_password(key, api_key.to_string()).ok().unwrap();
    if verify_key {
        let user_result = db.get_user(username).await;

        match user_result {
            Ok(Some(user)) => Ok(Json(user)),
            Ok(None) => {
                let result_string = "User not found".to_string();
                Err(Json(Db(Thrown(result_string))))
            }
            Err(err) => Err(Json(err)),
        }
    } else {
        Err(Json(Db(Thrown("Api key is invalid".to_string()))))
    }
}

#[get("/delete_user/<username>?<key>")]
pub async fn delete_user(
    username: String,
    key: String,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<String, Json<Error>> {
    let verify_key = verify_password(key, api_key.to_string()).ok().unwrap();
    if verify_key {
        let delete_result = db.delete_user(username.clone()).await;

        match delete_result {
            Ok(_) => Ok("User deleted".to_string()),
            Err(err) => Err(Json(err)),
        }
    } else {
        Err(Json(Db(Thrown("Api key is invalid".to_string()))))
    }
}

#[post("/email_login", data = "<credentials>")]
pub async fn email_login(
    credentials: Json<EmailLoginIn>,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<Json<EmailLoginInSuccess>, Json<Error>> {
    let verify_key = verify_password(credentials.api_key.clone(), api_key.to_string())
        .ok()
        .unwrap();
    if verify_key {
        let login_result = db.email_login(credentials.into_inner()).await;
        match login_result {
            Ok(login_success) => Ok(Json(login_success)),
            Err(err) => Err(Json(err)),
        }
    } else {
        Err(Json(Db(Thrown("Api key is invalid".to_string()))))
    }
}

#[get("/")]
pub fn root() -> &'static str {
    "Welcome to the Rust Auth Server created by PyDev19"
}
