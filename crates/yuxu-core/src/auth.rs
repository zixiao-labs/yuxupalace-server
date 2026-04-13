use anyhow::{Context, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user UUID
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,         // expiration timestamp
    pub iat: usize,         // issued at
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("failed to hash password: {}", e))?;
    Ok(hash.to_string())
}

pub fn verify_password(hash: &str, password: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("invalid password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn create_jwt(
    user_id: &Uuid,
    username: &str,
    is_admin: bool,
    secret: &str,
) -> Result<String> {
    let now = Utc::now().timestamp() as usize;
    let expiration = now + 7 * 24 * 60 * 60; // 7 days

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        is_admin,
        exp: expiration,
        iat: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("failed to create JWT token")?;

    Ok(token)
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .context("invalid or expired JWT token")?;

    Ok(token_data.claims)
}
