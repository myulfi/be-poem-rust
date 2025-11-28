use crate::models::command::role::MasterRoleMenu;
use crate::models::common::{DataResponse, PaginatedResponse};
use crate::schema::{tbl_mt_role, tbl_mt_role_menu};
use crate::utils::common::{self, validation_error_response};
use crate::{
    db::DbPool,
    models::command::role::{EntryMasterRole, MasterRole},
    models::common::Pagination,
};
// use bigdecimal::BigDecimal;
// use bigdecimal::num_bigint::BigInt;
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

    let mut query = tbl_mt_role::table.into_boxed();
    if let Some(ref term) = pagination.search {
        query = query.filter(tbl_mt_role::nm.ilike(format!("%{}%", term)));
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
        let mut query = tbl_mt_role::table.into_boxed();
        if let Some(ref term) = pagination.search {
            query = query.filter(tbl_mt_role::nm.ilike(format!("%{}%", term)));
        }

        match (pagination.sort.as_deref(), pagination.dir.as_deref()) {
            (Some("name"), Some("desc")) => query = query.order(tbl_mt_role::nm.desc()),
            (Some("name"), _) => query = query.order(tbl_mt_role::nm.asc()),
            (Some("createdDate"), Some("desc")) => {
                query = query.order(tbl_mt_role::dt_created.desc())
            }
            (Some("createdDate"), _) => query = query.order(tbl_mt_role::dt_created.asc()),
            _ => {}
        }

        let data = query
            .offset(start)
            .limit(length)
            .load::<MasterRole>(conn)
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
        // Err(common::error_message(StatusCode::NOT_FOUND, "Not Found"))
    }
}

#[handler]
pub fn get(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(role_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let master_role = tbl_mt_role::table
        .filter(tbl_mt_role::id.eq(role_id))
        .first::<MasterRole>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse { data: master_role }))
}

#[handler]
pub fn add(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Json(entry_master_role): Json<EntryMasterRole>,
) -> poem::Result<impl IntoResponse> {
    if let Err(e) = entry_master_role.validate() {
        return Err(validation_error_response(e));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;
    let max_id: Option<i16> = tbl_mt_role::table
        .select(diesel::dsl::max(tbl_mt_role::id))
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
        let master_role = MasterRole {
            id: next_id,
            nm: entry_master_role.nm,
            dscp: entry_master_role.dscp,
            is_del: 0,
            created_by: jwt_auth.claims.id,
            dt_created: Utc::now().naive_utc(),
            updated_by: None,
            dt_updated: None,
            version: 0,
        };

        let inserted = diesel::insert_into(tbl_mt_role::table)
            .values(&master_role)
            .get_result::<MasterRole>(conn)
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
    Path(role_id): Path<i16>,
    Json(mut entry_master_role): Json<EntryMasterRole>,
) -> poem::Result<impl IntoResponse> {
    if let Err(e) = entry_master_role.validate() {
        return Err(validation_error_response(e));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    entry_master_role.version = entry_master_role.version + 1;

    let updated = diesel::update(
        tbl_mt_role::table
            .filter(tbl_mt_role::id.eq(role_id))
            .filter(tbl_mt_role::version.eq(&entry_master_role.version - 1)),
    )
    // .set(&update)
    .set((
        &entry_master_role,
        tbl_mt_role::updated_by.eq(Some(jwt_auth.claims.id)),
        tbl_mt_role::dt_updated.eq(Some(Utc::now().naive_utc())),
    ))
    .get_result::<MasterRole>(conn)
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
    _: crate::auth::middleware::JwtAuth,
    Path(role_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    match diesel::delete(tbl_mt_role::table.filter(tbl_mt_role::id.eq(role_id))).execute(conn) {
        Ok(affected_rows) => {
            if affected_rows > 0 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(common::error_message(
                    StatusCode::NOT_FOUND,
                    "information.notFound",
                ))
            }
        }
        Err(e) => {
            eprintln!("Deleting error: {}", e);
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            ));
        }
    }
}

#[handler]
pub fn menu_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(role_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let data = tbl_mt_role_menu::table
        .filter(tbl_mt_role_menu::mt_role_id.eq(role_id))
        .load::<MasterRoleMenu>(conn)
        .map_err(|e| {
            eprintln!("Loading error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;
    Ok(Json(DataResponse { data }))
}

#[handler]
pub fn menu_update(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Path(role_id): Path<i16>,
    Json(menu_ids): Json<Vec<i16>>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    diesel::update(tbl_mt_role_menu::table.filter(tbl_mt_role_menu::mt_role_id.eq(role_id)))
        .set((tbl_mt_role_menu::is_del.eq(1),))
        .execute(conn)
        .map_err(|e| {
            eprintln!("Updating error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    let master_role_menu: Vec<MasterRoleMenu> = menu_ids
        .into_iter()
        .map(|menu_id| MasterRoleMenu {
            id: common::generate_id(),
            mt_role_id: role_id,
            mt_menu_id: menu_id,
            is_del: 0,
            created_by: jwt_auth.claims.id,
            dt_created: Utc::now().naive_utc(),
            updated_by: None,
            dt_updated: None,
            version: 0,
        })
        .collect();

    diesel::insert_into(tbl_mt_role_menu::table)
        .values(&master_role_menu)
        .execute(conn)
        .map_err(|e| {
            eprintln!("Inserting error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    diesel::delete(
        tbl_mt_role_menu::table.filter(
            tbl_mt_role_menu::mt_role_id
                .eq(role_id)
                .and(tbl_mt_role_menu::is_del.eq(1)),
        ),
    )
    .execute(conn)
    .map_err(|e| {
        eprintln!("Inserting error: {}", e);
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.internalServerError",
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
