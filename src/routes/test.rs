use crate::models::common::{DataResponse, PaginatedResponse};
use crate::schema::tbl_example_template::dsl::*;
use crate::utils::common;
use crate::{
    db::DbPool,
    models::common::Pagination,
    // schema::*,
    models::example_template::{ExampleTemplate, NewExampleTemplate, UpdateExampleTemplate},
};
use bigdecimal::BigDecimal;
use bigdecimal::num_bigint::BigInt;
use chrono::Utc;
use diesel::prelude::*;
use poem::IntoResponse;
use poem::web::Query;
use poem::{
    Route, get, handler,
    http::StatusCode,
    web::{Json, Path},
};

#[handler]
pub fn example_template_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::Middleware,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<ExampleTemplate>>, poem::Error> {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10);

    let mut query = tbl_example_template.into_boxed();
    if let Some(ref term) = pagination.search {
        query = query.filter(nm.ilike(format!("%{}%", term)));
    }

    let conn = &mut pool.get().unwrap();

    let total: i64 = query
        .count()
        .get_result(conn)
        .expect("Failed to count rows");

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
            .unwrap();
        Ok(Json(PaginatedResponse { total, data }))
    } else {
        Err(common::error_message(StatusCode::NOT_FOUND, "Not Found"))
    }
}

#[handler]
pub fn get_example_template_id(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::Middleware,
    Path(example_template_id): Path<i64>,
) -> Result<Json<DataResponse<ExampleTemplate>>, poem::Error> {
    let conn = &mut pool.get().unwrap();

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
    user: crate::auth::middleware::Middleware,
    Json(example_template): Json<NewExampleTemplate>,
    // ) -> Json<ExampleTemplate> {
) -> Result<Json<DataResponse<ExampleTemplate>>, poem::Error> {
    let new_template = ExampleTemplate {
        id: common::generate_id(),
        nm: example_template.nm,
        dscp: example_template.dscp,
        val: Some(99),
        amt: Some(BigDecimal::new(BigInt::from(9999), 2)),
        dt: Some(Utc::now().date_naive()),
        foreign_id: None,
        is_active: 0,
        is_del: 0,
        created_by: user.claims.username,
        dt_created: Utc::now().naive_utc(),
        updated_by: None,
        dt_updated: None,
        version: 0,
    };
    let conn = &mut pool.get().unwrap();
    let example_template = diesel::insert_into(tbl_example_template)
        .values(&new_template)
        .get_result(conn)
        // .map(Json)
        // .unwrap()
        .map_err(|e| {
            eprintln!("Diesel insert error: {:?}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to insert template",
            )
        })?;

    Ok(Json(DataResponse {
        data: example_template,
    }))
}

#[handler]
pub fn update_example_template(
    pool: poem::web::Data<&DbPool>,
    user: crate::auth::middleware::Middleware,
    Path(example_template_id): Path<i64>,
    Json(mut update): Json<UpdateExampleTemplate>,
) -> Result<Json<DataResponse<ExampleTemplate>>, poem::Error> {
    let conn = &mut pool.get().map_err(|e| {
        eprintln!("DB pool error: {:?}", e);
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database connection failed",
        )
    })?;

    update.version = update.version + 1;

    let updated = diesel::update(
        tbl_example_template
            .filter(id.eq(example_template_id))
            .filter(version.eq(&update.version - 1)),
    )
    // .set(&update)
    .set((
        &update,
        updated_by.eq(Some(user.claims.username.clone())),
        dt_updated.eq(Some(Utc::now().naive_utc())),
    ))
    .get_result::<ExampleTemplate>(conn)
    .map_err(|e| {
        eprintln!("Diesel insert error: {:?}", e);
        common::error_message(StatusCode::NOT_FOUND, "Failed to update template")
    })?;

    Ok(Json(DataResponse { data: updated }))
}

#[handler]
pub fn delete_example_template(
    pool: poem::web::Data<&DbPool>,
    Path(example_template_id): Path<i64>,
) -> Result<impl IntoResponse, poem::Error> {
    let conn = &mut pool.get().unwrap();

    match diesel::delete(tbl_example_template.filter(id.eq(example_template_id))).execute(conn) {
        Ok(affected_rows) => {
            if affected_rows == 0 {
                Err(common::error_message(
                    StatusCode::NOT_FOUND,
                    "No template found",
                ))
            } else {
                Ok(StatusCode::NO_CONTENT) // 204
            }
        }
        Err(e) => {
            eprintln!("Diesel delete error: {:?}", e);
            Err(common::error_message(
                StatusCode::NOT_FOUND,
                "Failed to delete template",
            ))
        }
    }
}

pub fn test_routes() -> Route {
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
