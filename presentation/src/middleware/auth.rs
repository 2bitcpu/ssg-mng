use axum::{
    RequestExt,
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use std::sync::Arc;

use application::{UseCaseModule, model::member::MemberSigninResponseDto};

#[derive(Clone)]
pub struct AuthMember {
    pub member: MemberSigninResponseDto,
}

impl<S> FromRequestParts<S> for AuthMember
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Self>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

pub async fn auth_guard(
    State(module): State<Arc<dyn UseCaseModule>>,
    mut request: Request,
    next: Next,
) -> axum::response::Result<Response> {
    let bearer = request
        .extract_parts::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let token = bearer.token();

    let member = module
        .auth()
        .authenticate(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let auth_member = AuthMember { member };
    request.extensions_mut().insert(auth_member);

    Ok(next.run(request).await)
}
