use crate::errors::error::AppError;
use crate::model::member::{
    MemberSigninRequestDto, MemberSigninResponseDto, MemberSignupRequestDto,
    MemberSignupResponseDto,
};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use config::CONFIG;
use domain::Repositories;
use std::sync::Arc;

#[allow(dead_code)]
pub struct AuthUseCase {
    repositories: Arc<dyn Repositories>,
}

impl AuthUseCase {
    pub fn new(repositories: Arc<dyn Repositories>) -> Self {
        Self { repositories }
    }

    pub async fn signup(
        &self,
        dto: &MemberSignupRequestDto,
    ) -> Result<MemberSignupResponseDto, AppError> {
        if !CONFIG.security.allow_signup {
            return Err(AppError::Forbidden());
        }

        if dto.password != dto.confirm_password {
            return Err(AppError::BadRequest(
                "Password confirmation does not match".to_string(),
            ));
        }

        if self
            .repositories
            .member()
            .find(&dto.account)
            .await?
            .is_some()
        {
            return Err(AppError::DataConflict(format!(
                "The account '{}' is already registered.",
                dto.account
            )));
        }

        let entity = dto.to_member_entity().await?;

        self.repositories.member().create(&entity).await?;
        self.repositories.member().commit().await?;

        Ok(MemberSignupResponseDto::from(entity))
    }

    pub async fn signin(
        &self,
        dto: &MemberSigninRequestDto,
    ) -> Result<MemberSigninResponseDto, AppError> {
        let entity = self.repositories.member().find(&dto.account).await?;
        if entity.is_none() {
            tracing::debug!("account not found: {}", dto.account);
            return Err(AppError::Unauthorized());
        }

        let mut entity = entity.unwrap();
        if !entity.is_busy(CONFIG.security.update_interval) {
            tracing::debug!("server busy");
            return Err(AppError::ServerBusy());
        }

        if entity.is_locked(CONFIG.security.lock_threshold, CONFIG.security.lock_seconds) {
            tracing::debug!("account locked");
            return Err(AppError::AccountLocked());
        }

        tracing::debug!(
            "password plain: {}, hash: {}",
            dto.password,
            entity.password
        );
        if !async_argon2::verify(dto.password.clone(), entity.password.clone()).await? {
            tracing::debug!("password mismatch");
            entity.signin_failed();
            self.repositories.member().edit(&entity).await?;
            self.repositories.member().commit().await?;
            return Err(AppError::Unauthorized());
        }

        let claims = simple_jwt::Claims::new(
            &entity.account,
            &CONFIG.security.issuer,
            CONFIG.security.expire,
        );
        let jti = claims.jti.clone();

        let token = simple_jwt::encode(&claims, &CONFIG.security.secret)
            .map_err(|e| AppError::Unexpected(e.into()))?;

        entity.signup_success(&jti);
        let result = self
            .repositories
            .member()
            .edit(&entity)
            .await?
            .ok_or_else(|| AppError::Inconsistent("member not found".to_string()))?;
        self.repositories.member().commit().await?;

        let response = MemberSigninResponseDto::new(result, token);
        Ok(response)
    }

    pub async fn signout(
        &self,
        bearer: Option<TypedHeader<Authorization<Bearer>>>,
    ) -> Result<(), AppError> {
        tracing::debug!("signout bearer: {:?}", bearer);
        let token = match bearer {
            Some(TypedHeader(Authorization(b))) => b.token().to_string(),
            None => return Ok(()),
        };
        tracing::debug!("signout token: {}", token);

        let claims = match simple_jwt::decode(
            &token,
            &CONFIG.security.issuer,
            &CONFIG.security.secret,
            true,
        ) {
            Ok(claims) => claims,
            Err(_) => return Ok(()),
        };
        tracing::debug!("signout claims: {:?}", claims);

        let mut entity = match self.repositories.member().find(&claims.sub).await? {
            Some(entity) => entity,
            None => return Ok(()),
        };
        tracing::debug!("signout entity: {:?}", entity);

        if entity.jti.is_none() {
            return Ok(());
        }

        if !entity.is_busy(CONFIG.security.update_interval) {
            return Err(AppError::ServerBusy());
        }
        tracing::debug!("signout verify OK");

        entity.signout();
        let _ = self
            .repositories
            .member()
            .edit(&entity)
            .await?
            .ok_or_else(|| AppError::Inconsistent("member not found".to_string()))?;
        self.repositories.member().commit().await?;

        Ok(())
    }

    pub async fn authenticate(&self, token: &str) -> Result<MemberSigninResponseDto, AppError> {
        let claims = simple_jwt::decode(
            &token,
            &CONFIG.security.issuer,
            &CONFIG.security.secret,
            true,
        )
        .map_err(|_| AppError::Unauthorized())?;

        let entity = self.repositories.member().find(&claims.sub).await?;
        if entity.is_none() {
            return Err(AppError::Unauthorized());
        }

        let entity = entity.unwrap();

        if let Some(jti) = entity.jti.clone() {
            if jti != claims.jti {
                return Err(AppError::Unauthorized());
            }
        } else {
            return Err(AppError::Unauthorized());
        }

        let response = MemberSigninResponseDto::new(entity, token.to_string());
        Ok(response)
    }
}
