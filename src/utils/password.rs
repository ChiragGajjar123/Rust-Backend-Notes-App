use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str, cost: u32) -> Result<String, bcrypt::BcryptError> {
    hash(password, cost)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}