use std::env;

use crate::auth::model::{AuthResponse, Claims, Login, UserAuthResponse};
use crate::db::DbPool;
use crate::models::user::User;
use crate::schema::tbl_user::dsl::*;
use crate::utils::common;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{EncodingKey, Header, encode};
use poem::post;
use poem::{Route, handler, http::StatusCode, web::Json};

fn create_token(
    usr: &str,
    secret_key_env: &str,
    duration_env: &str,
) -> Result<String, poem::Error> {
    let secret = env::var(secret_key_env)
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    let duration = env::var(duration_env)
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
        .parse::<i64>()
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    let exp = Utc::now()
        .checked_add_signed(Duration::days(duration))
        .ok_or_else(|| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
        .timestamp() as usize;

    let claims = Claims {
        username: usr.to_owned(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
}

fn build_auth_response(
    username_str: &str,
    nick_name: Option<String>,
) -> Result<AuthResponse, poem::Error> {
    let access_token = create_token(
        username_str,
        "JWT_ACCESS_TOKEN_SECRET",
        "JWT_ACCESS_TOKEN_EXPIRED",
    )?;

    let refresh_tkn = create_token(
        username_str,
        "JWT_REFRESH_TOKEN_SECRET",
        "JWT_REFRESH_TOKEN_EXPIRED",
    )?;

    Ok(AuthResponse {
        access_token,
        refresh_token: refresh_tkn,
        user: UserAuthResponse {
            nm: nick_name.unwrap_or("Guest".to_string()),
            role: vec![0, 1, 2, 4], // this can be customized later
        },
    })
}

#[handler]
pub fn generate_token(
    pool: poem::web::Data<&DbPool>,
    Json(login): Json<Login>,
) -> Result<Json<AuthResponse>, poem::Error> {
    let conn = &mut pool.get().unwrap();

    let user = tbl_user
        .filter(username.eq(&login.username))
        .filter(pass.eq(&login.password))
        .first::<User>(conn)
        .map_err(|err| match err {
            diesel::result::Error::NotFound => {
                common::error_message(StatusCode::UNAUTHORIZED, "Invalid username or password")
            }
            _ => poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        })?;

    Ok(Json(build_auth_response(&user.username, user.nick_nm)?))
    // Ok(Json("Login success".to_string()))
}

#[handler]
pub fn refresh_token(
    pool: poem::web::Data<&DbPool>,
    user_middleware: crate::auth::middleware::Middleware,
) -> Result<Json<AuthResponse>, poem::Error> {
    let conn = &mut pool.get().unwrap();
    let user = tbl_user
        .filter(username.eq(&user_middleware.claims.username))
        .first::<User>(conn)
        .map_err(|err| match err {
            diesel::result::Error::NotFound => {
                common::error_message(StatusCode::UNAUTHORIZED, "Invalid username")
            }
            _ => poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        })?;

    Ok(Json(build_auth_response(&user.username, user.nick_nm)?))
}

pub fn credential_routes() -> Route {
    Route::new()
        .at("/generate-token.json", post(generate_token))
        .at("/refresh-token.json", post(refresh_token))
}
