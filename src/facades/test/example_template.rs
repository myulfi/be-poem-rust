use crate::models::common::{DataResponse, PaginatedResponse};
use crate::schema::tbl_example_template;
use crate::utils::common::{
    self, parse_ids_from_string, validate_id, validate_ids, validation_error_response,
};
use crate::{
    db::DbPool,
    models::common::Pagination,
    models::example_template::{EntryExampleTemplate, ExampleTemplate},
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

    let mut query = tbl_example_template::table.into_boxed();
    if let Some(ref term) = pagination.search {
        query = query.filter(tbl_example_template::nm.ilike(format!("%{}%", term)));
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
        let mut query = tbl_example_template::table.into_boxed();
        if let Some(ref term) = pagination.search {
            query = query.filter(tbl_example_template::nm.ilike(format!("%{}%", term)));
        }

        match (pagination.sort.as_deref(), pagination.dir.as_deref()) {
            (Some("name"), Some("desc")) => query = query.order(tbl_example_template::nm.desc()),
            (Some("name"), _) => query = query.order(tbl_example_template::nm.asc()),
            (Some("createdDate"), Some("desc")) => {
                query = query.order(tbl_example_template::dt_created.desc())
            }
            (Some("createdDate"), _) => query = query.order(tbl_example_template::dt_created.asc()),
            _ => {}
        }

        let data = query
            .offset(start)
            .limit(length)
            .load::<ExampleTemplate>(conn)
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
    Path(example_template_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(example_template_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let example_template = tbl_example_template::table
        .filter(tbl_example_template::id.eq(example_template_id))
        .first::<ExampleTemplate>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse {
        data: example_template,
    }))
}

#[handler]
pub fn add(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Json(entry_example_template): Json<EntryExampleTemplate>,
) -> poem::Result<impl IntoResponse> {
    if let Err(e) = entry_example_template.validate() {
        return Err(validation_error_response(e));
    }

    let example_template = ExampleTemplate {
        id: common::generate_id(),
        nm: entry_example_template.nm,
        dscp: entry_example_template.dscp,
        val: entry_example_template.val,
        amt: entry_example_template.amt,
        dt: entry_example_template.dt,
        foreign_id: entry_example_template.foreign_id,
        is_active: 1,
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

    let inserted = diesel::insert_into(tbl_example_template::table)
        .values(&example_template)
        .get_result::<ExampleTemplate>(conn)
        .map_err(|e| {
            eprintln!("Inserting error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    Ok((StatusCode::CREATED, Json(DataResponse { data: inserted })))

    // let max_id: Option<i16> = tbl_ext_api::table
    //     .select(diesel::dsl::max(tbl_ext_api::id))
    //     .first(conn)
    //     .map_err(|e| {
    //         eprintln!("Loading error: {}", e);
    //         common::error_message(
    //             poem::http::StatusCode::INTERNAL_SERVER_ERROR,
    //             "information.internalServerError",
    //         )
    //     })?;

    // let next_id = max_id.unwrap_or(0).saturating_add(1);

    // if next_id < i16::MAX {
    //     let ext_api = ExternalApi {
    //         id: next_id,
    //         nm: entry_ext_api.nm,
    //         dscp: entry_ext_api.dscp,
    //         is_del: 0,
    //         created_by: jwt_auth.claims.username,
    //         dt_created: Utc::now().naive_utc(),
    //         updated_by: None,
    //         dt_updated: None,
    //         version: 0,
    //     };

    //     let inserted = diesel::insert_into(tbl_ext_api::table)
    //         .values(&ext_api)
    //         .get_result::<ExternalApi>(conn)
    //         .map_err(|e| {
    //             eprintln!("Inserting error: {}", e);
    //             common::error_message(
    //                 StatusCode::INTERNAL_SERVER_ERROR,
    //                 "information.internalServerError",
    //             )
    //         })?;

    //     Ok((StatusCode::CREATED, Json(DataResponse { data: inserted })))
    // } else {
    //     eprintln!("ID limit reached");
    //     Err(common::error_message(
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         "information.internalServerError",
    //     ))
    // }
}

#[handler]
pub fn update(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Path(example_template_id): Path<i64>,
    Json(mut entry_example_template): Json<EntryExampleTemplate>,
) -> poem::Result<impl IntoResponse> {
    validate_id(example_template_id)?;

    if let Err(e) = entry_example_template.validate() {
        return Err(validation_error_response(e));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    entry_example_template.version = entry_example_template.version + 1;

    let updated = diesel::update(
        tbl_example_template::table
            .filter(tbl_example_template::id.eq(example_template_id))
            .filter(tbl_example_template::version.eq(&entry_example_template.version - 1)),
    )
    // .set(&update)
    .set((
        &entry_example_template,
        tbl_example_template::updated_by.eq(Some(jwt_auth.claims.user_id)),
        tbl_example_template::dt_updated.eq(Some(Utc::now().naive_utc())),
    ))
    .get_result::<ExampleTemplate>(conn)
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
    Path(example_template_ids): Path<String>,
) -> poem::Result<impl IntoResponse> {
    validate_ids(&example_template_ids)?;
    let ids = parse_ids_from_string(&example_template_ids)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    match diesel::delete(tbl_example_template::table.filter(tbl_example_template::id.eq_any(ids)))
        .execute(conn)
    {
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
