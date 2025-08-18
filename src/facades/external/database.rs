use crate::models::common::{DataResponse, HeaderResponse, PaginatedResponse};
use crate::models::external::database::{
    EntryExternalDatabase, EntryQueryManual, ExternalDatabase, ExternalDatabaseQuery, QueryManual,
};
use crate::schema::tbl_ext_database::dsl::*;
use crate::schema::{tbl_ext_database_query, tbl_query_manual};
use crate::utils::common::{
    self, convert_to_count_query, extract_columns_info, rows_to_json, validate_id,
    validation_error_response,
};
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
use serde_json::{Value, json};
use tokio_postgres::NoTls;
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
    Path(ext_database_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let ext_database = tbl_ext_database
        .filter(id.eq(ext_database_id))
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

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let max_id: Option<i16> = tbl_ext_database
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
    Path(ext_database_id): Path<i16>,
    Json(mut entry_ext_database): Json<EntryExternalDatabase>,
) -> poem::Result<impl IntoResponse> {
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
    Path(ext_database_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    diesel::update(tbl_ext_database.filter(id.eq(ext_database_id)))
        .set((
            is_del.eq(1),
            updated_by.eq(Some(jwt_auth.claims.username.clone())),
            dt_updated.eq(Some(Utc::now().naive_utc())),
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

#[handler]
pub async fn connect(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let credential_ext_database: (String, String, String) = tbl_ext_database
        .filter(id.eq(ext_database_id))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let ext_database_connection = format!(
        "postgres://{}:{}@{}",
        credential_ext_database.0, credential_ext_database.1, credential_ext_database.2
    );

    let _ = tokio_postgres::connect(&ext_database_connection, NoTls)
        .await
        .map_err(|e| {
            eprintln!("Connection error: {}", e);
            common::error_message(StatusCode::BAD_GATEWAY, "information.connectionFailed")
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[handler]
pub async fn query_object_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i16>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let credential_ext_database: (String, String, String) = tbl_ext_database
        .filter(id.eq(ext_database_id))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let ext_database_connection = format!(
        "postgres://{}:{}@{}",
        credential_ext_database.0, credential_ext_database.1, credential_ext_database.2
    );

    let (client, connection) = tokio_postgres::connect(&ext_database_connection, NoTls)
        .await
        .map_err(|e| {
            eprintln!("Connection error: {}", e);
            common::error_message(StatusCode::BAD_GATEWAY, "information.connectionFailed")
        })?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let query = r#"
        SELECT
            objects.object_id,
            objects.object_name,
            objects.object_type
        FROM (
            SELECT
                pg_class.oid AS object_id,
                views.viewname AS object_name,
                'view' AS object_type
            FROM pg_catalog.pg_views views
            LEFT JOIN pg_class ON pg_class.relname = views.viewname
            WHERE views.schemaname = 'public'

            UNION ALL

            SELECT
                pg_class.oid AS object_id,
                tables.tablename AS object_name,
                'table' AS object_type
            FROM pg_catalog.pg_tables tables
            LEFT JOIN pg_class ON pg_class.relname = tables.tablename
            WHERE tables.schemaname = 'public'

            UNION ALL

            SELECT
                functions.pronamespace AS object_id,
                functions.proname AS object_name,
                'function' AS object_type
            FROM pg_catalog.pg_proc functions
            WHERE functions.pronamespace IN (
                SELECT oid FROM pg_catalog.pg_namespace WHERE nspname = 'public'
            )
        ) objects
        WHERE 1 = 1
        --{1}
        --ORDER BY {2}
    "#;

    if let Some(count_query) = convert_to_count_query(&query) {
        let row = match client.query_one(&count_query, &[]).await {
            Ok(row) => row,
            Err(_) => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "information.notFound",
                ));
            }
        };

        let total: i64 = row.get(0);
        if total > 0 {
            let paginated_query = format!(
                "{0} OFFSET {1} ROWS FETCH NEXT {2} ROWS ONLY",
                &query, start, length
            );
            let rows = client.query(&paginated_query, &[]).await.map_err(|e| {
                eprintln!("Query error: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

            let results = rows_to_json(&rows);

            Ok(Json(PaginatedResponse {
                total: total as i64,
                data: results,
            }))
        } else {
            Ok(Json(PaginatedResponse {
                total: 0,
                data: vec![],
            }))
        }
    } else {
        Ok(Json(PaginatedResponse {
            total: 0,
            data: vec![],
        }))
    }
}

#[handler]
pub fn query_whitelist_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i16>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);

    let mut query = tbl_ext_database_query::table.into_boxed();
    query = query.filter(tbl_ext_database_query::ext_database_id.eq(ext_database_id));
    if let Some(ref term) = pagination.search {
        query = query.filter(tbl_ext_database_query::dscp.ilike(format!("%{}%", term)));
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
        let mut query = tbl_ext_database_query::table.into_boxed();
        query = query.filter(tbl_ext_database_query::ext_database_id.eq(ext_database_id));
        if let Some(ref term) = pagination.search {
            query = query.filter(tbl_ext_database_query::dscp.ilike(format!("%{}%", term)));
        }

        match (pagination.sort.as_deref(), pagination.dir.as_deref()) {
            (Some("id"), Some("desc")) => query = query.order(tbl_ext_database_query::id.desc()),
            (Some("id"), _) => query = query.order(tbl_ext_database_query::id.asc()),
            (Some("createdDate"), Some("desc")) => {
                query = query.order(tbl_ext_database_query::dt_created.desc())
            }
            (Some("createdDate"), _) => {
                query = query.order(tbl_ext_database_query::dt_created.asc())
            }
            _ => {}
        }

        let data = query
            .offset(start)
            .limit(length)
            .load::<ExternalDatabaseQuery>(conn)
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
pub async fn query_manual_run(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i16>,
    Json(mut entry_manual_ext_database): Json<EntryQueryManual>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let credential_ext_database: (String, String, String) = tbl_ext_database
        .filter(id.eq(ext_database_id))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let ext_database_connection = format!(
        "postgres://{}:{}@{}",
        credential_ext_database.0, credential_ext_database.1, credential_ext_database.2
    );

    let (client, connection) = tokio_postgres::connect(&ext_database_connection, NoTls)
        .await
        .map_err(|e| {
            eprintln!("Connection error: {}", e);
            common::error_message(StatusCode::BAD_GATEWAY, "information.connectionFailed")
        })?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let query = format!("{0} LIMIT 1", entry_manual_ext_database.query);
    let rows = client.query(&query, &[]).await;

    let columns_info = match rows {
        Ok(rows) => {
            println!("{:?}", rows.len());
            extract_columns_info(&rows)
        }
        Err(e) => {
            let error_response = json!({
                "data": [
                    { "error": format!("{}", e) }
                ]
            });
            return Ok(Json(error_response));
        }
    };

    let inserted = diesel::insert_into(tbl_query_manual::table)
        .values(QueryManual {
            id: common::generate_id(),
            ext_database_id: ext_database_id,
            query: entry_manual_ext_database.query,
            created_by: jwt_auth.claims.username,
            dt_created: Utc::now().naive_utc(),
            updated_by: None,
            dt_updated: None,
            version: 0,
        })
        .get_result::<QueryManual>(conn)
        .map_err(|e| {
            eprintln!("Inserting error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    // Ok(Json(HeaderResponse {
    //     id: inserted.id,
    //     header: Value::Array(columns_info),
    // }))
    let success_response = json!({
        "id": inserted.id,
        "header": columns_info
    });

    Ok(Json(success_response))
    // Ok(Json(DataResponse { data: ext_database }))
}

#[handler]
pub async fn query_manual_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(query_manual_id): Path<i64>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    validate_id(query_manual_id)?;
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let query_manual_detail: (i16, String) = tbl_query_manual::table
        .filter(tbl_query_manual::id.eq(query_manual_id))
        .select((tbl_query_manual::ext_database_id, tbl_query_manual::query))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Not Found"))?;

    let credential_ext_database: (String, String, String) = tbl_ext_database
        .filter(id.eq(query_manual_detail.0))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let ext_database_connection = format!(
        "postgres://{}:{}@{}",
        credential_ext_database.0, credential_ext_database.1, credential_ext_database.2
    );

    let (client, connection) = tokio_postgres::connect(&ext_database_connection, NoTls)
        .await
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    // Jalankan koneksi di background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    if let Some(count_query) = convert_to_count_query(&query_manual_detail.1) {
        let row = match client.query_one(&count_query, &[]).await {
            Ok(row) => row,
            Err(_) => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "information.notFound",
                ));
            }
        };

        let total: i64 = row.get(0);
        if total > 0 {
            let paginated_query = format!(
                "{0} OFFSET {1} ROWS FETCH NEXT {2} ROWS ONLY",
                &query_manual_detail.1, start, length
            );
            let rows = client.query(&paginated_query, &[]).await.map_err(|e| {
                eprintln!("Query error: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

            let results = rows_to_json(&rows);
            Ok(Json(PaginatedResponse {
                total: total as i64,
                data: results,
            }))
        } else {
            Ok(Json(PaginatedResponse {
                total: 0,
                data: vec![],
            }))
        }
    } else {
        Ok(Json(PaginatedResponse {
            total: 0,
            data: vec![],
        }))
    }
}

#[handler]
pub async fn query_manual_all_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(query_manual_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(query_manual_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let query_manual_detail: (i16, String) = tbl_query_manual::table
        .filter(tbl_query_manual::id.eq(query_manual_id))
        .select((tbl_query_manual::ext_database_id, tbl_query_manual::query))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Not Found"))?;

    let credential_ext_database: (String, String, String) = tbl_ext_database
        .filter(id.eq(query_manual_detail.0))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let ext_database_connection = format!(
        "postgres://{}:{}@{}",
        credential_ext_database.0, credential_ext_database.1, credential_ext_database.2
    );

    let (client, connection) = tokio_postgres::connect(&ext_database_connection, NoTls)
        .await
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    // Jalankan koneksi di background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = client
        .query(&query_manual_detail.1, &[])
        .await
        .map_err(|e| {
            eprintln!("Query error: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    let results = rows_to_json(&rows);
    Ok(Json(DataResponse { data: results }))
}

#[handler]
pub async fn query_exact_object_run(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path((ext_database_id, entity_name)): Path<(i16, String)>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let credential_ext_database: (String, String, String) = tbl_ext_database
        .filter(id.eq(ext_database_id))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let ext_database_connection = format!(
        "postgres://{}:{}@{}",
        credential_ext_database.0, credential_ext_database.1, credential_ext_database.2
    );

    let (client, connection) = tokio_postgres::connect(&ext_database_connection, NoTls)
        .await
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let query = format!("SELECT * FROM {0} LIMIT 1", entity_name);

    let rows = client.query(&query, &[]).await.map_err(|e| {
        eprintln!("Query error: {}", e);
        poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let columns_info = extract_columns_info(&rows);
    Ok(Json(DataResponse {
        data: Value::Array(columns_info),
    }))
}

#[handler]
pub async fn query_exact_object_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path((ext_database_id, entity_name)): Path<(i16, String)>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let credential_ext_database: (String, String, String) = tbl_ext_database
        .filter(id.eq(ext_database_id))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let ext_database_connection = format!(
        "postgres://{}:{}@{}",
        credential_ext_database.0, credential_ext_database.1, credential_ext_database.2
    );

    let (client, connection) = tokio_postgres::connect(&ext_database_connection, NoTls)
        .await
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let query = format!("SELECT * FROM {0}", entity_name);
    if let Some(count_query) = convert_to_count_query(&query) {
        let row = match client.query_one(&count_query, &[]).await {
            Ok(row) => row,
            Err(_) => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "information.notFound",
                ));
            }
        };

        let total: i64 = row.get(0);
        if total > 0 {
            let paginated_query = format!(
                "{0} OFFSET {1} ROWS FETCH NEXT {2} ROWS ONLY",
                &query, start, length
            );
            let rows = client.query(&paginated_query, &[]).await.map_err(|e| {
                eprintln!("Query error: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

            let results = rows_to_json(&rows);
            Ok(Json(PaginatedResponse {
                total: total as i64,
                data: results,
            }))
        } else {
            Ok(Json(PaginatedResponse {
                total: 0,
                data: vec![],
            }))
        }
    } else {
        Ok(Json(PaginatedResponse {
            total: 0,
            data: vec![],
        }))
    }
}

#[handler]
pub async fn query_exact_whitelist_run(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_query_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let ext_database_query: (i16, String) = tbl_ext_database_query::table
        .filter(tbl_ext_database_query::id.eq(ext_database_query_id))
        .filter(tbl_ext_database_query::is_del.eq(0))
        .select((
            tbl_ext_database_query::ext_database_id,
            tbl_ext_database_query::query,
        ))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let credential_ext_database: (String, String, String) = tbl_ext_database
        .filter(id.eq(ext_database_query.0))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let ext_database_connection = format!(
        "postgres://{}:{}@{}",
        credential_ext_database.0, credential_ext_database.1, credential_ext_database.2
    );

    let (client, connection) = tokio_postgres::connect(&ext_database_connection, NoTls)
        .await
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    // Jalankan koneksi di background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = client
        .query(&ext_database_query.1, &[])
        .await
        .map_err(|e| {
            eprintln!("Query error: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    let columns_info = extract_columns_info(&rows);
    Ok(Json(DataResponse {
        data: Value::Array(columns_info),
    }))
}

#[handler]
pub async fn query_exact_whitelist_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_query_id): Path<i64>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_query_id)?;
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let ext_database_query: (i16, String) = tbl_ext_database_query::table
        .filter(tbl_ext_database_query::id.eq(ext_database_query_id))
        .filter(tbl_ext_database_query::is_del.eq(0))
        .select((
            tbl_ext_database_query::ext_database_id,
            tbl_ext_database_query::query,
        ))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let credential_ext_database: (String, String, String) = tbl_ext_database
        .filter(id.eq(ext_database_query.0))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let ext_database_connection = format!(
        "postgres://{}:{}@{}",
        credential_ext_database.0, credential_ext_database.1, credential_ext_database.2
    );

    let (client, connection) = tokio_postgres::connect(&ext_database_connection, NoTls)
        .await
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    if let Some(count_query) = convert_to_count_query(&ext_database_query.1) {
        let row = match client.query_one(&count_query, &[]).await {
            Ok(row) => row,
            Err(_) => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "information.notFound",
                ));
            }
        };

        let total: i64 = row.get(0);
        if total > 0 {
            let paginated_query = format!(
                "{0} OFFSET {1} ROWS FETCH NEXT {2} ROWS ONLY",
                &ext_database_query.1, start, length
            );

            let rows = client.query(&paginated_query, &[]).await.map_err(|e| {
                eprintln!("Query error: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

            let results = rows_to_json(&rows);
            Ok(Json(PaginatedResponse {
                total: total as i64,
                data: results,
            }))
        } else {
            Ok(Json(PaginatedResponse {
                total: 0,
                data: vec![],
            }))
        }
    } else {
        Ok(Json(PaginatedResponse {
            total: 0,
            data: vec![],
        }))
    }
}
