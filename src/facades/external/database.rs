use crate::models::common::{DataResponse, PaginatedResponse};
use crate::models::external::database::{EntryExternalDatabase, ExternalDatabase};
use crate::schema::tbl_ext_database;
use crate::utils::common::{self, parse_pagination, validate_id, validation_error_response};
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
    let (start, length) = parse_pagination(&pagination);

    let mut query = tbl_ext_database::table.into_boxed();
    if let Some(ref term) = pagination.search {
        query = query.filter(tbl_ext_database::cd.ilike(format!("%{}%", term)));
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
        let mut query = tbl_ext_database::table.into_boxed();
        if let Some(ref term) = pagination.search {
            query = query.filter(tbl_ext_database::cd.ilike(format!("%{}%", term)));
        }

        match (pagination.sort.as_deref(), pagination.dir.as_deref()) {
            (Some("code"), Some("desc")) => query = query.order(tbl_ext_database::cd.desc()),
            (Some("code"), _) => query = query.order(tbl_ext_database::cd.asc()),
            (Some("createdDate"), Some("desc")) => {
                query = query.order(tbl_ext_database::dt_created.desc())
            }
            (Some("createdDate"), _) => query = query.order(tbl_ext_database::dt_created.asc()),
            _ => {}
        }

        let data = query
            .offset(start)
            .limit(length)
            .load::<ExternalDatabase>(conn)
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
    Path(ext_database_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let ext_database = tbl_ext_database::table
        .filter(tbl_ext_database::id.eq(ext_database_id))
        .first::<ExternalDatabase>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse { data: ext_database }))
}

#[handler]
pub fn add(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Json(entry_ext_database): Json<EntryExternalDatabase>,
) -> poem::Result<impl IntoResponse> {
    if let Err(e) = entry_ext_database.validate() {
        return Err(validation_error_response(e));
    }

    let ext_database = ExternalDatabase {
        id: common::generate_id(),
        cd: entry_ext_database.cd,
        dscp: entry_ext_database.dscp,
        ext_server_id: entry_ext_database.ext_server_id,
        mt_database_type_id: entry_ext_database.mt_database_type_id,
        ip: entry_ext_database.ip,
        port: entry_ext_database.port,
        username: entry_ext_database.username,
        password: entry_ext_database.password,
        db_name: entry_ext_database.db_name,
        // db_connection: entry_ext_database.db_connection,
        is_use_page: entry_ext_database.is_use_page,
        is_lock: entry_ext_database.is_lock,
        is_del: 0,
        created_by: jwt_auth.claims.id,
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

    let inserted = diesel::insert_into(tbl_ext_database::table)
        .values(&ext_database)
        .get_result::<ExternalDatabase>(conn)
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
    Path(ext_database_id): Path<i64>,
    Json(mut entry_ext_database): Json<EntryExternalDatabase>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_id)?;

    if let Err(e) = entry_ext_database.validate() {
        return Err(validation_error_response(e));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    entry_ext_database.version = entry_ext_database.version + 1;

    let updated = diesel::update(
        tbl_ext_database::table
            .filter(tbl_ext_database::id.eq(ext_database_id))
            .filter(tbl_ext_database::version.eq(&entry_ext_database.version - 1)),
    )
    .set((
        &entry_ext_database,
        tbl_ext_database::updated_by.eq(Some(jwt_auth.claims.id)),
        tbl_ext_database::dt_updated.eq(Some(Utc::now().naive_utc())),
    ))
    .get_result::<ExternalDatabase>(conn)
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
    Path(ext_database_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    diesel::update(tbl_ext_database::table.filter(tbl_ext_database::id.eq(ext_database_id)))
        .set((
            tbl_ext_database::is_del.eq(1),
            tbl_ext_database::updated_by.eq(Some(jwt_auth.claims.id)),
            tbl_ext_database::dt_updated.eq(Some(Utc::now().naive_utc())),
        ))
        .get_result::<ExternalDatabase>(conn)
        .map_err(|e| {
            eprintln!("Soft Deleting error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}
