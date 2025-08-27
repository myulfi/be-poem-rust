use crate::db::DbPool;
use crate::models::common::DataResponse;
use crate::schema::{tbl_ext_server, tbl_mt_database_type, tbl_mt_server_type};
use crate::utils::common;
use diesel::prelude::*;
use poem::IntoResponse;
use poem::{handler, http::StatusCode, web::Json};

#[handler]
pub fn database_type(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let mt_database_type = tbl_mt_database_type::table
        .filter(tbl_mt_database_type::is_del.eq(0))
        .select((tbl_mt_database_type::id, tbl_mt_database_type::nm))
        .load::<(i16, String)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse {
        data: mt_database_type
            .into_iter()
            .map(|(key, value)| serde_json::json!({ "key": key, "value": value }))
            .collect::<Vec<serde_json::Value>>(),
    }))
}

#[handler]
pub fn server_type(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let mt_database_type = tbl_mt_server_type::table
        .filter(tbl_mt_server_type::is_del.eq(0))
        .select((
            tbl_mt_server_type::id,
            tbl_mt_server_type::nm,
            tbl_mt_server_type::icon,
        ))
        .load::<(i16, String, String)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse {
        data: mt_database_type
            .into_iter()
            .map(|(key, value, icon)| serde_json::json!({ "key": key, "value": value, "icon": icon }))
            .collect::<Vec<serde_json::Value>>(),
    }))
}

#[handler]
pub fn external_server(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let ext_server = tbl_ext_server::table
        .filter(tbl_ext_server::is_del.eq(0))
        .select((tbl_ext_server::id, tbl_ext_server::cd))
        .load::<(i16, String)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse {
        data: ext_server
            .into_iter()
            .map(|(key, value)| serde_json::json!({ "key": key, "value": value }))
            .collect::<Vec<serde_json::Value>>(),
    }))
}
