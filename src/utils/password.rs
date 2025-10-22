use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, PasswordHash, rand_core::OsRng};
use crate::errors::{AppError, AppResult};

pub struct PasswordService;

impl PasswordService {
    pub fn hash_password(password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::PasswordHash(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {

        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| AppError::PasswordHash(e.to_string()))?;

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
