use crate::models::login::Claims;
use crate::utils::common;
use jsonwebtoken::{DecodingKey, Validation, decode};
use poem::{FromRequest, Request, RequestBody, http::StatusCode};
use std::{env, future::Future};

pub struct AuthenticatedUser {
    pub claims: Claims,
}

impl<'a> FromRequest<'a> for AuthenticatedUser {
    fn from_request(
        req: &'a Request,
        _body: &mut RequestBody,
        //) -> Pin<Box<dyn Future<Output = PoemResult<Self>> + Send + 'a>> {
    ) -> impl Future<Output = Result<Self, poem::Error>> + Send {
        Box::pin(async move {
            let auth_header = req
                .headers()
                .get("authorization")
                .ok_or_else(|| {
                    common::error_message(StatusCode::UNAUTHORIZED, "Missing authorization header")
                })?
                .to_str()
                .map_err(|_| {
                    common::error_message(StatusCode::UNAUTHORIZED, "Invalid authorization header")
                })?;

            let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
                common::error_message(StatusCode::UNAUTHORIZED, "Missing Bearer prefix")
            })?;

            let secret = env::var("JWT_ACCESS_TOKEN_SECRET").map_err(|_| {
                common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Missing JWT secret")
            })?;

            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| {
                common::error_message(StatusCode::UNAUTHORIZED, "Invalid or expired token")
            })?;

            Ok(AuthenticatedUser {
                claims: token_data.claims,
            })
        })
    }
}
