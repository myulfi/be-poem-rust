use crate::models::common::{DataResponse, PaginatedResponse};
use crate::models::external::database::{EntryExternalDatabase, ExternalDatabase};
use crate::schema::tbl_ext_database::dsl::*;
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

    let mut query = tbl_ext_database.into_boxed();
    if let Some(ref term) = pagination.search {
        query = query.filter(cd.ilike(format!("%{}%", term)));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    let total: i64 = match query.count().get_result(conn) {
        Ok(count) => count,
        Err(_) => {
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Not Found",
            ));
        }
    };

    if total > 0 {
        let mut query = tbl_ext_database.into_boxed();
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
            .load::<ExternalDatabase>(conn)
            .map_err(|_| {
                common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load data")
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
    Path(ext_database_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    let ext_database = tbl_ext_database
        .filter(id.eq(ext_database_id))
        .first::<ExternalDatabase>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Not Found"))?;

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

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    let max_id: Option<i16> = tbl_ext_database
        .select(diesel::dsl::max(id))
        .first(conn)
        .map_err(|_| {
            common::error_message(
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get max ID",
            )
        })?;

    let next_id = max_id.unwrap_or(0).saturating_add(1);

    if next_id < i16::MAX {
        let ext_database = ExternalDatabase {
            id: next_id,
            cd: entry_ext_database.cd,
            dscp: entry_ext_database.dscp,
            mt_database_type_id: entry_ext_database.mt_database_type_id,
            username: entry_ext_database.username,
            password: entry_ext_database.password,
            db_connection: entry_ext_database.db_connection,
            is_lock: entry_ext_database.is_lock,
            is_del: 0,
            created_by: jwt_auth.claims.username,
            dt_created: Utc::now().naive_utc(),
            updated_by: None,
            dt_updated: None,
            version: 0,
        };

        let inserted = diesel::insert_into(tbl_ext_database)
            .values(&ext_database)
            .get_result::<ExternalDatabase>(conn)
            .map_err(|_| {
                common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Failed to insert")
            })?;

        Ok((StatusCode::CREATED, Json(DataResponse { data: inserted })))
    } else {
        Err(common::error_message(
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            "ID limit reached",
        ))
    }
}

#[handler]
pub fn update(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i16>,
    Json(mut entry_ext_database): Json<EntryExternalDatabase>,
) -> poem::Result<impl IntoResponse> {
    if let Err(e) = entry_ext_database.validate() {
        return Err(validation_error_response(e));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    entry_ext_database.version = entry_ext_database.version + 1;

    let updated = diesel::update(
        tbl_ext_database
            .filter(id.eq(ext_database_id))
            .filter(version.eq(&entry_ext_database.version - 1)),
    )
    .set((
        &entry_ext_database,
        updated_by.eq(Some(jwt_auth.claims.username.clone())),
        dt_updated.eq(Some(Utc::now().naive_utc())),
    ))
    .get_result::<ExternalDatabase>(conn)
    .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Failed to update"))?;

    Ok(Json(DataResponse { data: updated }))
}

#[handler]
pub fn delete(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    diesel::update(tbl_ext_database.filter(id.eq(ext_database_id)))
        .set((
            is_del.eq(1),
            updated_by.eq(Some(jwt_auth.claims.username.clone())),
            dt_updated.eq(Some(Utc::now().naive_utc())),
        ))
        .get_result::<ExternalDatabase>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Failed to update"))?;

    Ok(StatusCode::NO_CONTENT)
}
