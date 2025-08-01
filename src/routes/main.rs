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
    web::{Json},
};
// use validator::Validate;

#[handler]
pub fn menu_list(
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

pub fn routes() -> Route {
    Route::new().nest("/menu.json", get(menu_list))
}
