use crate::models::common::{DataResponse, PaginatedResponse};
use crate::models::external::server::{EntryExternalServer, ExternalServer};
use crate::schema::tbl_ext_server::dsl::*;
use crate::utils::common::{self, validation_error_response};
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

    let mut query = tbl_ext_server.into_boxed();
    if let Some(ref term) = pagination.search {
        query = query.filter(cd.ilike(format!("%{}%", term)));
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
        let mut query = tbl_ext_server.into_boxed();
        if let Some(ref term) = pagination.search {
            query = query.filter(cd.ilike(format!("%{}%", term)));
        }

        match (pagination.sort.as_deref(), pagination.dir.as_deref()) {
            (Some("code"), Some("desc")) => query = query.order(cd.desc()),
            (Some("code"), _) => query = query.order(cd.asc()),
            (Some("createdDate"), Some("desc")) => query = query.order(dt_created.desc()),
            (Some("createdDate"), _) => query = query.order(dt_created.asc()),
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
    Path(ext_server_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let ext_server = tbl_ext_server
        .filter(id.eq(ext_server_id))
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

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let max_id: Option<i16> = tbl_ext_server
        .select(diesel::dsl::max(id))
        .first(conn)
        .map_err(|e| {
            eprintln!("Loading error: {}", e);
            common::error_message(
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    let next_id = max_id.unwrap_or(0).saturating_add(1);

    if next_id < i16::MAX {
        let ext_server = ExternalServer {
            id: next_id,
            cd: entry_ext_server.cd,
            dscp: entry_ext_server.dscp,
            ip: entry_ext_server.ip,
            port: entry_ext_server.port,
            username: entry_ext_server.username,
            password: entry_ext_server.password,
            private_key: entry_ext_server.private_key,
            is_lock: entry_ext_server.is_lock,
            is_del: 0,
            created_by: jwt_auth.claims.username,
            dt_created: Utc::now().naive_utc(),
            updated_by: None,
            dt_updated: None,
            version: 0,
        };

        let inserted = diesel::insert_into(tbl_ext_server)
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
    } else {
        eprintln!("ID limit reached");
        Err(common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.internalServerError",
        ))
    }
}

#[handler]
pub fn update(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i16>,
    Json(mut entry_ext_server): Json<EntryExternalServer>,
) -> poem::Result<impl IntoResponse> {
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
        tbl_ext_server
            .filter(id.eq(ext_server_id))
            .filter(version.eq(&entry_ext_server.version - 1)),
    )
    .set((
        &entry_ext_server,
        updated_by.eq(Some(jwt_auth.claims.username.clone())),
        dt_updated.eq(Some(Utc::now().naive_utc())),
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
    Path(ext_server_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    diesel::update(tbl_ext_server.filter(id.eq(ext_server_id)))
        .set((
            is_del.eq(1),
            updated_by.eq(Some(jwt_auth.claims.username.clone())),
            dt_updated.eq(Some(Utc::now().naive_utc())),
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
