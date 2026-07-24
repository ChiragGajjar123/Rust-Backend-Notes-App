use crate::errors::AppError;

pub async fn hash_password(password: String, cost: u32) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        bcrypt::hash(password, cost).map_err(AppError::PasswordError)
    })
    .await?
}

pub async fn verify_password(password: String, hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        bcrypt::verify(password, &hash).map_err(AppError::PasswordError)
    })
    .await?
}