use crate::errors::error::ApiError;
use application::{
    UseCaseModule,
    model::member::{
        MemberSigninRequestDto, MemberSigninResponseDto, MemberSignupRequestDto,
        MemberSignupResponseDto,
    },
};
use axum::Json;
use axum::extract::State;
use axum_valid::Valid;
use std::sync::Arc;

pub async fn signup(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Valid(Json(dto)): Valid<Json<MemberSignupRequestDto>>,
) -> Result<Json<MemberSignupResponseDto>, ApiError> {
    let res = usecases.auth().signup(&dto).await?;
    Ok(Json(res))
}

pub async fn signin(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Json(dto): Json<MemberSigninRequestDto>,
) -> Result<Json<MemberSigninResponseDto>, ApiError> {
    let res = usecases.auth().signin(&dto).await?;
    Ok(Json(res))
}

use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

pub async fn signout(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<(), ApiError> {
    tracing::debug!("signout bearer: {:?}", bearer);
    usecases.auth().signout(bearer).await?;
    Ok(())
}
