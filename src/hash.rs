use argon2::{Argon2, PasswordHasher, PasswordVerifier, PasswordHash};
use argon2::password_hash::{SaltString, rand_core::OsRng, Error};

pub fn generate_salt() -> SaltString {
   SaltString::generate(&mut OsRng)
}

pub fn hash_password(password: String, salt: SaltString) -> Result<String, Error> {
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

pub fn verify_password(password: String, password_hash: String) -> Result<bool, Error> {
    let hash = PasswordHash::new(&password_hash)?;
    dbg!(hash.clone());
    Ok(Argon2::default().verify_password(password.as_bytes(), &hash).is_ok())
} 
