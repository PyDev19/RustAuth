use crate::hash::*;
use crate::models::*;
use std::borrow::Borrow;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::error::Db::Thrown;
use surrealdb::opt::auth::Root;
use surrealdb::{Error, Error::Db, Surreal};

pub struct Database {
    pub client: Surreal<Client>,
    pub name_space: String,
    pub db_name: String,
}

impl Database {
    pub async fn new() -> Result<Self, Error> {
        let client = Surreal::new::<Ws>("127.0.0.1:8080").await?;
        client
            .signin(Root {
                username: "root",
                password: "root",
            })
            .await?;
        client.use_ns("my_ns").use_db("my_db").await.unwrap();
        Ok(Database {
            client,
            name_space: String::from("my_ns"),
            db_name: String::from("my_db"),
        })
    }

    pub async fn check_duplicate_email(&self, email: String) -> Result<bool, Error> {
        let query = format!("SELECT * FROM Users WHERE email = '{}'", email);
        let result = self.client.query(query).await;

        match result {
            Ok(mut result_set) => {
                let created: Option<User> = result_set.take(0)?;
                Ok(!created.is_none())
            }
            Err(err) => Err(err),
        }
    }

    pub async fn check_duplicate_username(&self, username: String) -> Result<bool, Error> {
        let query = format!("SELECT * FROM Users WHERE username = '{}'", username);
        let result = self.client.query(query).await;
        match result {
            Ok(mut result_set) => {
                let created: Option<User> = result_set.take(0)?;
                Ok(!created.is_none())
            }
            Err(err) => Err(err),
        }
    }

    pub async fn signup(&self, user: SignUp) -> Result<Option<User>, Error> {
        let is_duplicate_email = self.check_duplicate_email(user.email.clone()).await;
        match is_duplicate_email {
            Ok(duplicate) => {
                if duplicate {
                    return Err(Db(Thrown("Email already in use".to_string())));
                }
            }
            Err(err) => return Err(err),
        };

        let is_duplicate_username = self.check_duplicate_username(user.username.clone()).await;
        match is_duplicate_username {
            Ok(duplicate) => {
                if duplicate {
                    return Err(Db(Thrown("Username already taken".to_string())));
                }
            }
            Err(err) => return Err(err),
        };

        let salt = generate_salt();
        let password_hash = hash_password(user.password.clone(), salt.clone()).ok();

        let user_data: User = User {
            email: user.email.clone(),
            username: user.username.clone(),
            password: password_hash.unwrap(),
            logged_in: false,
        };

        let created_user = self.client.create("Users").content(user_data).await;
        match created_user {
            Ok(user) => Ok(user.into_iter().next()),
            Err(err) => Err(err),
        }
    }

    pub async fn email_login(
        &self,
        credentials: EmailLoginIn,
    ) -> Result<EmailLoginInSuccess, Error> {
        let query = format!(
            "SELECT * FROM Users WHERE email='{}'",
            credentials.email.clone()
        );
        let result = self.client.query(query).await;
        match result {
            Ok(mut result_set) => {
                let user: Option<User> = result_set.take(0)?;
                match user {
                    Some(user) => {
                        let verify_password =
                            verify_password(credentials.password.clone(), user.password.clone())
                                .ok()
                                .unwrap();
                        dbg!(verify_password);
                        if verify_password {
                            let logged_in_query = format!(
                                "UPDATE Users SET logged_in = true WHERE email='{}'",
                                user.email.clone()
                            );
                            match self.client.query(logged_in_query).await {
                                Ok(_) => Ok(EmailLoginInSuccess {
                                    email: user.email.clone(),
                                    username: user.username.clone(),
                                }),
                                Err(err) => Err(err),
                            }
                        } else {
                            Err(Db(Thrown("Password is incorrent try again".to_string())))
                        }
                    }
                    None => Err(Db(Thrown("Email is incorrent try again".to_string()))),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_user(&self, username: String) -> Result<Option<User>, Error> {
        let query = format!("SELECT * FROM Users WHERE username = '{}'", username);
        let mut result = self.client.query(query).await?;
        let created: Option<User> = result.take(0)?;
        Ok(created)
    }

    pub async fn delete_user(&self, username: String) -> Result<Option<User>, Error> {
        let get_user_result = self.get_user(username.clone()).await;
        match get_user_result {
            Ok(Some(_user)) => {
                let query = format!("DELETE Users WHERE username = '{}'", username);
                let mut result = self.client.query(query).await?;
                let deleted_user: Option<User> = result.take(0)?;
                dbg!(deleted_user.borrow());
                Ok(deleted_user)
            }
            Ok(None) => Err(Db(Thrown("User not found".to_string()))),
            Err(err) => Err(err),
        }
    }
}
