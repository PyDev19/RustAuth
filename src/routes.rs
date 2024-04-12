use crate::{
    database::Database,
    hash::verify_password,
    models::{EmailLogin, LoginSuccess, SignUp, User, UsernameLogin},
};
use core::future::Future;
use rocket::serde::json::Json;
use rocket::State;
use surrealdb::{error::Db::Thrown, Error, Error::Db};

async fn verify_api_key<T, F, U>(
    key: String,
    api_key: &State<String>,
    action: T,
) -> Result<U, Json<Error>>
where
    T: FnOnce() -> F,
    F: Future<Output = Result<U, Error>>,
{
    let verify_key_result = verify_password(key, api_key.to_string());
    match verify_key_result {
        Ok(verify_key) => {
            if verify_key {
                action().await.map_err(Json)
            } else {
                Err(Json(Db(Thrown("Api key is invalid".to_string()))))
            }
        }
        Err(_) => Err(Json(Db(Thrown("Api key is required".to_string())))),
    }
}

#[post("/signup", data = "<user>")]
pub async fn signup(
    user: Json<SignUp>,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<Json<User>, Json<Error>> {
    verify_api_key(user.api_key.clone(), api_key, || async {
        let created_user = db.signup(user.into_inner()).await;

        match created_user {
            Ok(result) => match result {
                Some(user) => Ok(Json(user)),
                None => Err(Db(Thrown("An error occured".to_string()))),
            },
            Err(err) => Err(err),
        }
    })
    .await
}

#[get("/get_user/<username>?<key>")]
pub async fn get_user(
    username: String,
    key: String,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<Json<User>, Json<Error>> {
    verify_api_key(key, api_key, || async {
        let user_result = db.get_user(username).await;
        match user_result {
            Ok(Some(user)) => Ok(Json(user)),
            Ok(None) => {
                let result_string = "User not found".to_string();
                Err(Db(Thrown(result_string)))
            }
            Err(err) => Err(err),
        }
    })
    .await
}

#[get("/delete_user/<username>?<key>")]
pub async fn delete_user(
    username: String,
    key: String,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<String, Json<Error>> {
    verify_api_key(key, api_key, || async {
        let delete_result = db.delete_user(username.clone()).await;

        match delete_result {
            Ok(_) => Ok("User deleted".to_string()),
            Err(err) => Err(err),
        }
    })
    .await
}

#[post("/email_login", data = "<credentials>")]
pub async fn email_login(
    credentials: Json<EmailLogin>,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<Json<LoginSuccess>, Json<Error>> {
    verify_api_key(credentials.api_key.clone(), api_key, || async {
        let login_result = db.email_login(credentials.into_inner()).await;
        match login_result {
            Ok(login_success) => Ok(Json(login_success)),
            Err(err) => Err(err),
        }
    })
    .await
}

#[get("/signout/<username>?<key>")]
pub async fn signout(
    username: String,
    key: String,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<String, Json<Error>> {
    verify_api_key(key, api_key, || async {
        let signout_result = db.signout(username).await;
        match signout_result {
            Ok(success) => Ok(success),
            Err(err) => Err(err),
        }
    })
    .await
}

#[post("/username_login", data = "<credentials>")]
pub async fn username_login(
    credentials: Json<UsernameLogin>,
    db: &State<Database>,
    api_key: &State<String>,
) -> Result<Json<LoginSuccess>, Json<Error>> {
    verify_api_key(credentials.api_key.clone(), api_key, || async {
        let login_result = db.username_login(credentials.into_inner()).await;
        match login_result {
            Ok(login_success) => Ok(Json(login_success)),
            Err(err) => Err(err),
        }
    })
    .await
}

// #[get("/account_recovery/<username>?<key>")]
// pub async fn account_recovery(
//     username: String,
//     key: String,
//     api_key: &State<String>,
//     db: &State<Database>,
// ) -> Result<String, Json<Error>> {
//     verify_api_key(key, api_key, || async {
//         match db.check_code(username.clone()).await {
//             Ok(user) => {
//                 match send_code_mail(
//                     user.recovery_code.unwrap().to_string(),
//                     username,
//                     user.email,
//                 ) {
//                     Ok(_) => Ok("Email with authentication code sent to user's email".to_string()),
//                     Err(err) => Err(Error::Db(Thrown(err.to_string()))),
//                 }
//             }
//             Err(e) => Err(e),
//         }
//     })
//     .await
// }

#[get("/")]
pub fn root() -> &'static str {
    "Welcome to the Rust Auth Server created by PyDev19"
}
