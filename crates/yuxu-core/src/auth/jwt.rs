use crate::{Error, Result};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub is_admin: bool,
    pub exp: u64,
    pub iat: u64,
}

#[derive(Clone)]
pub struct JwtService {
    encode: EncodingKey,
    decode: DecodingKey,
    ttl_seconds: i64,
}

impl JwtService {
    pub fn new(secret: &[u8], ttl_seconds: i64) -> Self {
        Self {
            encode: EncodingKey::from_secret(secret),
            decode: DecodingKey::from_secret(secret),
            ttl_seconds,
        }
    }

    pub fn issue(&self, user_id: &str, username: &str, is_admin: bool) -> Result<String> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::seconds(self.ttl_seconds);
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            is_admin,
            iat: now.timestamp() as u64,
            exp: exp.timestamp() as u64,
        };
        Ok(encode(&Header::default(), &claims, &self.encode)?)
    }

    pub fn verify(&self, token: &str) -> Result<Claims> {
        let data = decode::<Claims>(token, &self.decode, &Validation::default())
            .map_err(|e| Error::Unauthorized(format!("invalid token: {e}")))?;
        Ok(data.claims)
    }
}
