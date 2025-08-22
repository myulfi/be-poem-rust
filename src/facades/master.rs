use crate::models::common::DataResponse;
use crate::models::master::database_type::DatabaseType;
use crate::schema::{tbl_ext_server, tbl_mt_database_type};
use crate::utils::common;
use crate::{db::DbPool, models::external::server::ExternalServer};
use diesel::ExpressionMethods;
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
        .load::<DatabaseType>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse {
        data: mt_database_type,
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
        .load::<ExternalServer>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse { data: ext_server }))
}
