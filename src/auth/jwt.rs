use std::env;

use crate::auth::model::{AuthResponse, Claims, Login, UserAuthResponse};
use crate::db::DbPool;
use crate::models::common::DataResponse;
use crate::models::user::User;
use crate::models::user_role::UserRole;
use crate::schema::tbl_user;
use crate::schema::tbl_user_role;
use crate::utils::common;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{EncodingKey, Header, encode};
use poem::IntoResponse;
use poem::{handler, http::StatusCode, web::Json};

fn create_token(
    id: i64,
    roles: Option<&Vec<i16>>,
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
        id,
        role: roles.cloned(),
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
    conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>,
    user: User,
) -> Result<AuthResponse, poem::Error> {
    let user_role = tbl_user_role::table
        .filter(tbl_user_role::user_id.eq(user.id))
        .load::<UserRole>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Not Found"))?;
    let roles: Vec<i16> = user_role.into_iter().map(|r| r.mt_role_id).collect();

    let access_token = create_token(
        user.id,
        Some(&roles),
        "JWT_ACCESS_TOKEN_SECRET",
        "JWT_ACCESS_TOKEN_EXPIRED",
    )?;

    let refresh_tkn = create_token(
        user.id,
        None,
        "JWT_REFRESH_TOKEN_SECRET",
        "JWT_REFRESH_TOKEN_EXPIRED",
    )?;

    Ok(AuthResponse {
        access_token,
        refresh_token: refresh_tkn,
        user: UserAuthResponse {
            nm: user.nick_nm.unwrap_or_else(|| "Guest".to_string()),
            role: roles,
        },
    })
}

#[handler]
pub fn generate_token(
    pool: poem::web::Data<&DbPool>,
    Json(login): Json<Login>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    let user = tbl_user::table
        .filter(tbl_user::username.eq(&login.username))
        .filter(tbl_user::pass.eq(&login.password))
        .first::<User>(conn)
        .map_err(|err| match err {
            diesel::result::Error::NotFound => {
                common::error_message(StatusCode::UNAUTHORIZED, "Invalid username or password")
            }
            _ => poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        })?;

    Ok(Json(DataResponse {
        data: build_auth_response(conn, user)?,
    }))
}

#[handler]
pub fn refresh_token(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    let user = tbl_user::table
        .filter(tbl_user::id.eq(jwt_auth.claims.id))
        .first::<User>(conn)
        .map_err(|err| match err {
            diesel::result::Error::NotFound => {
                common::error_message(StatusCode::UNAUTHORIZED, "Invalid username")
            }
            _ => poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        })?;

    Ok(Json(DataResponse {
        data: build_auth_response(conn, user)?,
    }))
}
