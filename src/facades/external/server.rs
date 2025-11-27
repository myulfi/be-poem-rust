use crate::models::common::{DataResponse, PaginatedResponse};
use crate::models::external::server::{EntryExternalServer, ExternalServer};
use crate::schema::tbl_ext_server;
use crate::utils::common::{self, validate_id, validation_error_response};
use crate::{db::DbPool, models::common::Pagination};
use chrono::Utc;
use diesel::prelude::*;
use poem::IntoResponse;
use poem::web::Query;
use poem::{
    handler,
    http::StatusCode,
    web::{Json, Path},
};
use validator::Validate;

#[handler]
pub fn list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);

    let mut query = tbl_ext_server::table.into_boxed();
    if let Some(ref term) = pagination.search {
        query = query.filter(tbl_ext_server::cd.ilike(format!("%{}%", term)));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let total: i64 = match query.count().get_result(conn) {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Counting error: {}", e);
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            ));
        }
    };

    if total > 0 {
        let mut query = tbl_ext_server::table.into_boxed();
        if let Some(ref term) = pagination.search {
            query = query.filter(tbl_ext_server::cd.ilike(format!("%{}%", term)));
        }

        match (pagination.sort.as_deref(), pagination.dir.as_deref()) {
            (Some("code"), Some("desc")) => query = query.order(tbl_ext_server::cd.desc()),
            (Some("code"), _) => query = query.order(tbl_ext_server::cd.asc()),
            (Some("createdDate"), Some("desc")) => {
                query = query.order(tbl_ext_server::dt_created.desc())
            }
            (Some("createdDate"), _) => query = query.order(tbl_ext_server::dt_created.asc()),
            _ => {}
        }

        let data = query
            .offset(start)
            .limit(length)
            .load::<ExternalServer>(conn)
            .map_err(|e| {
                eprintln!("Loading error: {}", e);
                common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "information.internalServerError",
                )
            })?;
        Ok(Json(PaginatedResponse { total, data }))
    } else {
        Ok(Json(PaginatedResponse {
            total: 0,
            data: vec![],
        }))
    }
}

#[handler]
pub fn get(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let ext_server = tbl_ext_server::table
        .filter(tbl_ext_server::id.eq(ext_server_id))
        .first::<ExternalServer>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse { data: ext_server }))
}

#[handler]
pub fn add(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Json(entry_ext_server): Json<EntryExternalServer>,
) -> poem::Result<impl IntoResponse> {
    if let Err(e) = entry_ext_server.validate() {
        return Err(validation_error_response(e));
    }

    let ext_server = ExternalServer {
        id: common::generate_id(),
        cd: entry_ext_server.cd,
        dscp: entry_ext_server.dscp,
        mt_server_type_id: entry_ext_server.mt_server_type_id,
        ip: entry_ext_server.ip,
        port: entry_ext_server.port,
        username: entry_ext_server.username,
        password: entry_ext_server.password,
        private_key: entry_ext_server.private_key,
        is_lock: 1,
        is_del: 0,
        created_by: jwt_auth.claims.user_id,
        dt_created: Utc::now().naive_utc(),
        updated_by: None,
        dt_updated: None,
        version: 0,
    };

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let inserted = diesel::insert_into(tbl_ext_server::table)
        .values(&ext_server)
        .get_result::<ExternalServer>(conn)
        .map_err(|e| {
            eprintln!("Inserting error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    Ok((StatusCode::CREATED, Json(DataResponse { data: inserted })))
}

#[handler]
pub fn update(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Json(mut entry_ext_server): Json<EntryExternalServer>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    if let Err(e) = entry_ext_server.validate() {
        return Err(validation_error_response(e));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    entry_ext_server.version = entry_ext_server.version + 1;

    let updated = diesel::update(
        tbl_ext_server::table
            .filter(tbl_ext_server::id.eq(ext_server_id))
            .filter(tbl_ext_server::version.eq(&entry_ext_server.version - 1)),
    )
    .set((
        &entry_ext_server,
        tbl_ext_server::updated_by.eq(Some(jwt_auth.claims.user_id)),
        tbl_ext_server::dt_updated.eq(Some(Utc::now().naive_utc())),
    ))
    .get_result::<ExternalServer>(conn)
    .map_err(|e| {
        eprintln!("Updating error: {}", e);
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.internalServerError",
        )
    })?;

    Ok(Json(DataResponse { data: updated }))
}

#[handler]
pub fn delete(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    diesel::update(tbl_ext_server::table.filter(tbl_ext_server::id.eq(ext_server_id)))
        .set((
            tbl_ext_server::is_del.eq(1),
            tbl_ext_server::updated_by.eq(Some(jwt_auth.claims.user_id)),
            tbl_ext_server::dt_updated.eq(Some(Utc::now().naive_utc())),
        ))
        .get_result::<ExternalServer>(conn)
        .map_err(|e| {
            eprintln!("Soft Deleting error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}
