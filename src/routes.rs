use rocket::serde::json::Json;
use surrealdb::Error;
use surrealdb::error::Db;
use rocket::State;
use crate::models::*;
use crate::database::Database;
use crate::hash::*;

#[post("/signup", data = "<user>")]
pub async fn signup(user: Json<SignUp>, db: &State<Database>) -> Result<Result<Result<Json<User>, Json<String>>, String>, Json<Error>> {
    let created_user = db.signup(user.into_inner()).await;
    
    match created_user {
        Ok(result) => {
            match result {
                Ok(user) => {
                    match user {
                        Some(user) => Ok(Ok(Ok(Json(user)))),
                        None => Ok(Ok(Err(Json("An error occured".to_string()))))
                    }
                },
                Err(err) => Ok(Err(err)) 
            } 
        }
        Err(err) => Err(Json(err)),
    }
}

#[get("/get_user/<username>?<key>")]
pub async fn get_user(username: String, key: String, db: &State<Database>, api_key: &State<String>) -> Result<Result<Json<User>, String>, Json<Error>> {
    let verify_key = verify_password(key, api_key.to_string()).ok().unwrap();
    if verify_key {
        let user_result = db.get_user(username).await;

        match user_result {
            Ok(Some(user)) => Ok(Ok(Json(user))),
            Ok(None) => {
                let result_string = "User not found".to_string();
                Ok(Err(result_string))
            }
            Err(err) => Err(Json(err)),
        }
    } else {
        Err(Json(Error::Db(Db::Thrown("Api key is invalid".to_string()))))
    }
}

#[get("/delete_user/<username>")]
pub async fn delete_user(username: String, db: &State<Database>) -> Result<String, Json<Error>> {
    let delete_result = db.delete_user(username.clone()).await;

    match delete_result {
        Ok(Ok(_deleted_user)) => {
            Ok("User deleted".to_string())
        }
        Ok(Err(error_string)) => {
            Ok(error_string)
        }
        Err(err) => Err(Json(err))
    }
}

#[post("/email_login", data = "<credentials>")]
pub async fn email_login(credentials: Json<EmailLoginIn>, db: &State<Database>) -> Result<Result<Json<EmailLoginInSuccess>, String>, Json<Error>> {
    let login_result = db.email_login(credentials.into_inner()).await;
    match login_result {
        Ok(no_err) => {
            match no_err {
                Ok(login_success) => {
                    Ok(Ok(Json(login_success)))
                }
                Err(err) => Ok(Err(err))
            }
        }
        Err(err) => Err(Json(err)) 
    }
} 

#[get("/")]
pub fn root() -> &'static str {
    "Welcome to the Rust Auth Server created by PyDev19"
}
