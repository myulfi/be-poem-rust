use std::env;

use crate::db::DbPool;
use crate::models::login::{AuthResponse, Claims, Login, User, UserAuthResponse};
use crate::schema::tbl_user::dsl::*;
use crate::utils::common;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use poem::http::HeaderValue;
use poem::{Request, post};
use poem::{Route, handler, http::StatusCode, web::Json};

fn extract_bearer_token(header: &HeaderValue) -> Result<&str, poem::Error> {
    let header_str = header
        .to_str()
        .map_err(|_| poem::Error::from_status(StatusCode::BAD_REQUEST))?;
    if let Some(token) = header_str.strip_prefix("Bearer ") {
        Ok(token)
    } else {
        Err(poem::Error::from_status(StatusCode::UNAUTHORIZED))
    }
}

fn create_token(
    usr: &str,
    secret_key_env: &str,
    // duration: Duration,
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

    let access_token = create_token(
        &user.username,
        "JWT_ACCESS_TOKEN_SECRET",
        "JWT_ACCESS_TOKEN_EXPIRED",
    )?;
    let refresh_tkn = create_token(
        &user.username,
        "JWT_REFRESH_TOKEN_SECRET",
        "JWT_REFRESH_TOKEN_EXPIRED",
    )?;

    Ok(Json(AuthResponse {
        access_token: access_token,
        refresh_token: refresh_tkn,
        user: UserAuthResponse {
            nm: user.nick_nm.unwrap_or("Guest".to_string()),
            role: vec![0, 1, 2, 4],
        },
    }))

    // Ok(Json("Login success".to_string()))
}

#[handler]
pub fn refresh_token(
    pool: poem::web::Data<&DbPool>,
    req: &Request,
) -> Result<Json<AuthResponse>, poem::Error> {
    let auth_header = req
        .headers()
        .get("authorization")
        .ok_or_else(|| poem::Error::from_status(StatusCode::UNAUTHORIZED))?;
    let token_str = extract_bearer_token(auth_header)?;
    let refresh_secret = env::var("JWT_REFRESH_TOKEN_SECRET")
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    let token_data = decode::<Claims>(
        &token_str,
        &DecodingKey::from_secret(refresh_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| common::error_message(StatusCode::UNAUTHORIZED, "Invalid Token"))?;
    let user_name = token_data.claims.username;

    let conn = &mut pool.get().unwrap();
    let user = tbl_user
        .filter(username.eq(&user_name))
        .first::<User>(conn)
        .map_err(|err| match err {
            diesel::result::Error::NotFound => {
                common::error_message(StatusCode::UNAUTHORIZED, "Invalid username")
            }
            _ => poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        })?;

    let access_token = create_token(
        &user_name,
        "JWT_ACCESS_TOKEN_SECRET",
        "JWT_ACCESS_TOKEN_EXPIRED",
    )?;
    let refresh_token: String = create_token(
        &user_name,
        "JWT_REFRESH_TOKEN_SECRET",
        "JWT_REFRESH_TOKEN_EXPIRED",
    )?;

    Ok(Json(AuthResponse {
        access_token: access_token,
        refresh_token: refresh_token,
        user: UserAuthResponse {
            nm: user.nick_nm.unwrap_or("Guest".to_string()),
            role: vec![0, 1, 2, 4],
        },
    }))
}

pub fn credential_routes() -> Route {
    Route::new()
        .at("/generate-token.json", post(generate_token))
        .at("/refresh-token.json", post(refresh_token))
}
