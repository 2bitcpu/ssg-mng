use crate::custom_validator::validation::{validate_account, validate_password};
use chrono::Utc;
use common::types::BoxError;
use domain::model::member::MemberEntity;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct MemberSignupRequestDto {
    #[validate(custom(function = "validate_account", message = "invalid account"))]
    pub account: String,
    #[validate(custom(
        function = "validate_password",
        message = "Password must be 8-64 chars, include uppercase, lowercase, number, and symbol"
    ))]
    pub password: String,
    pub confirm_password: String,
    #[validate(email)]
    pub email: Option<String>,
}

impl MemberSignupRequestDto {
    pub async fn to_member_entity(&self) -> Result<MemberEntity, BoxError> {
        let password_hash = async_argon2::hash(self.password.clone()).await?;
        Ok(MemberEntity {
            account: self.account.clone(),
            password: password_hash,
            jti: None,
            email: self.email.clone(),
            failed_attempts: 0,
            last_failed_at: None,
            last_signin_at: None,
            updated_at: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberSignupResponseDto {
    pub account: String,
    pub email: Option<String>,
}

impl From<MemberEntity> for MemberSignupResponseDto {
    fn from(entity: MemberEntity) -> Self {
        Self {
            account: entity.account,
            email: entity.email,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberSigninRequestDto {
    pub account: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberSigninResponseDto {
    pub account: String,
    pub email: Option<String>,
    pub token: String,
}

impl MemberSigninResponseDto {
    pub fn new(entity: MemberEntity, token: String) -> Self {
        Self {
            account: entity.account,
            email: entity.email,
            token,
        }
    }
}
