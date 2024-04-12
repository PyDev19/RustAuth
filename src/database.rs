use crate::{
    hash::{generate_salt, hash_password, verify_password},
    models::{EmailLogin, LoginSuccess, SignUp, User, UsernameLogin},
    settings::{DatabaseType, Settings},
};
use rand::Rng;
use surrealdb::{
    engine::local::{Db, RocksDb},
    engine::remote::ws::{Client, Ws},
    error::Db::Thrown,
    opt::{auth::Root, Config},
    {Error, Response, Surreal},
};

pub enum DbClient {
    Db(Surreal<Db>),
    Client(Surreal<Client>),
}

impl DbClient {
    async fn query(&self, query: String) -> Result<Response, Error> {
        match self {
            DbClient::Db(db) => db.query(query).await,
            DbClient::Client(client) => client.query(query).await,
        }
    }
}

pub struct Database {
    pub client: DbClient,
    pub name_space: String,
    pub db_name: String,
}

impl Database {
    pub async fn new(db_settings: Settings, root_password: String) -> Result<Self, Error> {
        match db_settings.clone().database_type.unwrap() {
            DatabaseType::Local => {
                let config = Config::default().strict().user(Root {
                    username: db_settings.root_user.clone().unwrap().as_str(),
                    password: root_password.as_str(),
                });
                let client =
                    Surreal::new::<RocksDb>((db_settings.database_endpoint.unwrap(), config))
                        .await?;
                client
                    .signin(Root {
                        username: db_settings.root_user.unwrap().as_str(),
                        password: root_password.as_str(),
                    })
                    .await?;
                client.use_ns("my_ns").use_db("my_db").await?;
                Ok(Database {
                    client: DbClient::Db(client),
                    name_space: String::from("my_ns"),
                    db_name: String::from("my_db"),
                })
            }
            DatabaseType::Remote => {
                let client = Surreal::new::<Ws>(db_settings.database_endpoint.unwrap()).await?;
                client
                    .signin(Root {
                        username: "root",
                        password: "root",
                    })
                    .await?;
                client.use_ns("my_ns").use_db("my_db").await.unwrap();
                Ok(Database {
                    client: DbClient::Client(client),
                    name_space: String::from("my_ns"),
                    db_name: String::from("my_db"),
                })
            }
        }
    }

    pub async fn check_duplicate_email(&self, email: String) -> Result<bool, Error> {
        let query = format!("SELECT * FROM Users WHERE email = '{email}'");
        let result = self.client.query(query).await;

        match result {
            Ok(mut result_set) => {
                let created: Option<User> = result_set.take(0)?;
                Ok(created.is_some())
            }
            Err(err) => Err(err),
        }
    }

    pub async fn check_duplicate_username(&self, username: String) -> Result<bool, Error> {
        let query = format!("SELECT * FROM Users WHERE username = '{username}'");
        let result = self.client.query(query).await;
        match result {
            Ok(mut result_set) => {
                let created: Option<User> = result_set.take(0)?;
                Ok(created.is_some())
            }
            Err(err) => Err(err),
        }
    }

    pub async fn signup(&self, user: SignUp) -> Result<Option<User>, Error> {
        let is_duplicate_email = self.check_duplicate_email(user.email.clone()).await;
        match is_duplicate_email {
            Ok(duplicate) => {
                if duplicate {
                    return Err(Error::Db(Thrown("Email already in use".to_string())));
                }
            }
            Err(err) => return Err(err),
        };

        let is_duplicate_username = self.check_duplicate_username(user.username.clone()).await;
        match is_duplicate_username {
            Ok(duplicate) => {
                if duplicate {
                    return Err(Error::Db(Thrown("Username already taken".to_string())));
                }
            }
            Err(err) => return Err(err),
        };

        let salt = generate_salt();
        let password_hash = hash_password(user.password.clone(), salt.clone()).ok();

        let query = format!(
            "CREATE Users SET email='{}', username='{}', password='{}', logged_in=false",
            user.email,
            user.username,
            password_hash.unwrap()
        );
        let mut result = self.client.query(query).await?;
        result.take(0)
    }

    pub async fn email_login(&self, credentials: EmailLogin) -> Result<LoginSuccess, Error> {
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
                        if verify_password {
                            if user.logged_in {
                                Err(Error::Db(Thrown("User already logged in".to_string())))
                            } else {
                                let logged_in_query = format!(
                                    "UPDATE Users SET logged_in = true WHERE email='{}'",
                                    user.email.clone()
                                );
                                match self.client.query(logged_in_query).await {
                                    Ok(_) => Ok(LoginSuccess {
                                        email: user.email.clone(),
                                        username: user.username.clone(),
                                    }),
                                    Err(err) => Err(err),
                                }
                            }
                        } else {
                            Err(Error::Db(Thrown(
                                "Email or Password is incorret try again".to_string(),
                            )))
                        }
                    }
                    None => Err(Error::Db(Thrown(
                        "Email or Password is incorret try again".to_string(),
                    ))),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn username_login(&self, credentials: UsernameLogin) -> Result<LoginSuccess, Error> {
        let query = format!(
            "SELECT * FROM Users WHERE username='{}'",
            credentials.username.clone()
        );
        let result = self.client.query(query).await;
        match result {
            Ok(mut result_) => {
                let user: Option<User> = result_.take(0)?;
                match user {
                    Some(user_) => {
                        let verify_password =
                            verify_password(credentials.password.clone(), user_.password.clone())
                                .ok()
                                .unwrap();
                        if verify_password {
                            if user_.logged_in {
                                Err(Error::Db(Thrown("User already logged in".to_string())))
                            } else {
                                let logged_in_query = format!(
                                    "UPDATE Users SET logged_in = true WHERE email='{}'",
                                    user_.email.clone()
                                );
                                match self.client.query(logged_in_query).await {
                                    Ok(_) => Ok(LoginSuccess {
                                        email: user_.email.clone(),
                                        username: user_.username.clone(),
                                    }),
                                    Err(err) => Err(err),
                                }
                            }
                        } else {
                            Err(Error::Db(Thrown(
                                "Username or Password is incorret try again".to_string(),
                            )))
                        }
                    }
                    None => Err(Error::Db(Thrown(
                        "Username or Password is incorret try again".to_string(),
                    ))),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_user(&self, username: String) -> Result<Option<User>, Error> {
        let query = format!("SELECT * FROM Users WHERE username = '{username}'");
        let mut result = self.client.query(query).await?;
        let created: Option<User> = result.take(0)?;
        Ok(created)
    }

    pub async fn delete_user(&self, username: String) -> Result<Option<User>, Error> {
        let get_user_result = self.get_user(username.clone()).await;
        match get_user_result {
            Ok(Some(_user)) => {
                let query = format!("DELETE Users WHERE username = '{username}'");
                let mut result = self.client.query(query).await?;
                let deleted_user: Option<User> = result.take(0)?;
                Ok(deleted_user)
            }
            Ok(None) => Err(Error::Db(Thrown("User not found".to_string()))),
            Err(err) => Err(err),
        }
    }

    pub async fn signout(&self, username: String) -> Result<String, Error> {
        let get_user_result = self.get_user(username.clone()).await;
        match get_user_result {
            Ok(Some(user)) => {
                if user.logged_in {
                    let query = format!(
                        "UPDATE Users SET logged_in=false WHERE username = '{username}'"
                    );
                    let mut result = self.client.query(query).await?;
                    let _deleted_user: Option<User> = result.take(0)?;
                    Ok("User successfully logged out".to_string())
                } else {
                    Err(Error::Db(Thrown("User is not logged in".to_string())))
                }
            }
            Ok(None) => Err(Error::Db(Thrown("User not found".to_string()))),
            Err(err) => Err(err),
        }
    }

    async fn add_code(&self, username: String, code: i32) -> Result<User, Error> {
        let query = format!("UPDATE Users SET recovery_code={code} WHERE username='{username}'");
        let mut result = self.client.query(query).await?;
        let recovery_user: Option<User> = result.take(0)?;
        Ok(recovery_user.unwrap())
    }

    pub async fn check_code(&self, username: String) -> Result<User, Error> {
        let user = self.get_user(username.clone()).await?;
        match user {
            Some(user_value) => {
                if user_value.recovery_code.clone().is_some() {
                    Err(Error::Db(Thrown(
                        "Account recovery code already exists on user".to_string(),
                    )))
                } else {
                    let code = rand::thread_rng().gen_range(100_000..1_000_000);
                    self.add_code(username, code).await
                }
            }
            None => Err(Error::Db(Thrown("User doesn't exists".to_string()))),
        }
    }
}
