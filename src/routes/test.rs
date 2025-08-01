use crate::models::common::{DataResponse, PaginatedResponse};
use crate::schema::tbl_example_template::dsl::*;
use crate::utils::common::{self, validate_id, validation_error_response};
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
    Route, get, handler,
    http::StatusCode,
    web::{Json, Path},
};
use validator::Validate;

#[handler]
pub fn example_template_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10);

    let mut query = tbl_example_template.into_boxed();
    if let Some(ref term) = pagination.search {
        query = query.filter(nm.ilike(format!("%{}%", term)));
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
        let mut query = tbl_example_template.into_boxed();
        if let Some(ref term) = pagination.search {
            query = query.filter(nm.ilike(format!("%{}%", term)));
        }

        match (pagination.sort.as_deref(), pagination.dir.as_deref()) {
            (Some("name"), Some("desc")) => query = query.order(nm.desc()),
            (Some("name"), _) => query = query.order(nm.asc()),
            (Some("createdDate"), Some("desc")) => query = query.order(dt_created.desc()),
            (Some("createdDate"), _) => query = query.order(dt_created.asc()),
            _ => {}
        }

        let data = query
            .offset(start)
            .limit(length)
            .load::<ExampleTemplate>(conn)
            .map_err(|_| {
                common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load data")
            })?;
        Ok(Json(PaginatedResponse { total, data }))
    } else {
        Err(common::error_message(StatusCode::NOT_FOUND, "Not Found"))
    }
}

#[handler]
pub fn get_example_template_id(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(example_template_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(example_template_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    let example_template = tbl_example_template
        .filter(id.eq(example_template_id))
        .first::<ExampleTemplate>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Not Found"))?;

    Ok(Json(DataResponse {
        data: example_template,
    }))
}

#[handler]
pub fn add_example_template(
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
        created_by: jwt_auth.claims.username,
        dt_created: Utc::now().naive_utc(),
        updated_by: None,
        dt_updated: None,
        version: 0,
    };

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    let inserted = diesel::insert_into(tbl_example_template)
        .values(&example_template)
        .get_result::<ExampleTemplate>(conn)
        .map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Failed to insert")
        })?;

    Ok((StatusCode::CREATED, Json(DataResponse { data: inserted })))
}

#[handler]
pub fn update_example_template(
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
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    entry_example_template.version = entry_example_template.version + 1;

    let updated = diesel::update(
        tbl_example_template
            .filter(id.eq(example_template_id))
            .filter(version.eq(&entry_example_template.version - 1)),
    )
    // .set(&update)
    .set((
        &entry_example_template,
        updated_by.eq(Some(jwt_auth.claims.username.clone())),
        dt_updated.eq(Some(Utc::now().naive_utc())),
    ))
    .get_result::<ExampleTemplate>(conn)
    .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Failed to update"))?;

    Ok(Json(DataResponse { data: updated }))
}

#[handler]
pub fn delete_example_template(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(example_template_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(example_template_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    match diesel::delete(tbl_example_template.filter(id.eq(example_template_id))).execute(conn) {
        Ok(affected_rows) => {
            if affected_rows == 0 {
                Err(common::error_message(
                    StatusCode::NOT_FOUND,
                    "No Data found",
                ))
            } else {
                Ok(StatusCode::NO_CONTENT)
            }
        }
        Err(_) => Err(common::error_message(
            StatusCode::NOT_FOUND,
            "Failed to update",
        )),
    }
}

pub fn routes() -> Route {
    Route::new()
        .at(
            "/example-template.json",
            get(example_template_list).post(add_example_template),
        )
        .at(
            "/:id/example-template.json",
            get(get_example_template_id)
                .patch(update_example_template)
                .delete(delete_example_template),
        )
}
