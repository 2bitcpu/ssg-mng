use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use tokio::task;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub async fn hash(password: String) -> Result<String, BoxError> {
    task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string())
    })
    .await?
}

pub async fn verify(password: String, hash: String) -> Result<bool, BoxError> {
    task::spawn_blocking(move || {
        let hash = PasswordHash::new(&hash)?;
        match Argon2::default().verify_password(password.as_bytes(), &hash) {
            Ok(()) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e.into()),
        }
    })
    .await?
}
