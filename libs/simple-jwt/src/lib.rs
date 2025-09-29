use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    pub sub: String, // subject (ユーザーの識別子)
    pub iss: String, // issuer (JWTの発行者)
    pub iat: i64,    // Issued At (発行日時)
    pub exp: i64,    // expiration time (トークンの有効期限)
    pub jti: String, // JWT ID (JWTの一意な識別子)
}

impl Claims {
    pub fn new(subject: &str, issuer: &str, duration_seconds: i64) -> Self {
        let current_time: DateTime<Utc> = Utc::now();
        Self {
            sub: subject.to_string(),
            iss: issuer.to_string(),
            iat: current_time.timestamp(),
            exp: current_time.timestamp() + duration_seconds,
            jti: uuid::Uuid::new_v4().to_string(),
        }
    }
}

pub fn encode(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.to_string().as_bytes()),
    )?)
}

pub fn decode(
    token: &str,
    issuer: &str,
    secret: &str,
    validate_exp: bool,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    if validate_exp {
        validation.leeway = 30;
        validation.validate_exp = true;
    } else {
        validation.validate_exp = false;
    }
    validation.set_issuer(&[issuer]);
    let claims: Claims = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.to_string().as_ref()),
        &validation,
    )?
    .claims;
    Ok(claims)
}
