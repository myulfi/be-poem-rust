use crate::models::common::{DataResponse, PaginatedResponse};
use crate::models::external::database::{
    EntryExternalDatabase, EntryQueryManual, ExternalDatabase, ExternalDatabaseQuery, QueryManual,
};
use crate::schema::tbl_ext_database::dsl::*;
use crate::schema::{tbl_ext_database_query, tbl_query_manual};
use crate::utils::common::{
    self, convert_to_count_query, extract_columns_info, extract_query_parts, is_only_comment,
    is_sql_type, rows_to_csv_string, rows_to_insert_query_string, rows_to_json,
    rows_to_json_string, rows_to_update_query_string, rows_to_xml_string, split_manual_query,
    validate_id, validation_error_response,
};
use crate::{db::DbPool, models::common::Pagination};
use chrono::Utc;
use diesel::prelude::*;
use futures::future::ok;
// use fancy_regex::Regex;
use poem::web::Query;
use poem::{IntoResponse, Result};
use poem::{
    handler,
    http::StatusCode,
    web::{Json, Path},
};
use serde_json::{Value, json};
use tokio_postgres::{Client, NoTls, Row};
use validator::Validate;

fn parse_pagination(pagination: &Pagination) -> (i64, i64) {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);
    (start, length)
}

fn get_ext_database_info(
    conn: &mut PgConnection,
    ext_database_id: i16,
) -> poem::Result<(String, String, String)> {
    tbl_ext_database
        .filter(id.eq(ext_database_id))
        .filter(is_del.eq(0))
        .select((username, password, db_connection))
        .first::<(String, String, String)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))
}

fn build_connection_string(usr: &str, pass: &str, db_conn: &str) -> String {
    format!("postgres://{}:{}@{}", usr, pass, db_conn)
}

async fn connect_to_external_database(connection_str: &str) -> poem::Result<Client> {
    let (client, connection) = tokio_postgres::connect(connection_str, NoTls)
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

    Ok(client)
}

//perlu di gabungkan
fn get_query_manual_detail(
    conn: &mut PgConnection,
    query_manual_id: i64,
) -> poem::Result<(i16, String)> {
    tbl_query_manual::table
        .filter(tbl_query_manual::id.eq(query_manual_id))
        .select((tbl_query_manual::ext_database_id, tbl_query_manual::query))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Not Found"))
}

async fn get_query_manual_client(
    conn: &mut PgConnection,
    query_manual_id: i64,
) -> poem::Result<(Client, String)> {
    let (ext_database_id, query_str) = get_query_manual_detail(conn, query_manual_id)?;
    let client = get_external_pg_client(conn, ext_database_id).await?;
    Ok((client, query_str))
}

async fn get_query_manual_row(
    conn: &mut PgConnection,
    query_manual_id: i64,
) -> poem::Result<(Vec<Row>, String)> {
    let (client, query_str) = get_query_manual_client(conn, query_manual_id).await?;
    let rows = client.query(&query_str, &[]).await.map_err(|e| {
        eprintln!("Query error: {}", e);
        poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    Ok((rows, query_str))
}

async fn get_external_pg_client(
    conn: &mut PgConnection,
    ext_database_id: i16,
) -> poem::Result<Client> {
    let (usr, pass, db_conn) = get_ext_database_info(conn, ext_database_id)?;
    let connection_str = build_connection_string(&usr, &pass, &db_conn);
    connect_to_external_database(&connection_str).await
}

async fn query_with_pagination(
    client: &Client,
    base_query: &str,
    start: i64,
    length: i64,
) -> poem::Result<PaginatedResponse<Value>> {
    if let Some(count_query) = convert_to_count_query(base_query) {
        let total: i64 = client
            .query_one(&count_query, &[])
            .await
            .map(|row| row.get(0))
            .map_err(|_| {
                common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "information.notFound")
            })?;

        let data = if total > 0 {
            let paginated_query = format!(
                "{} OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
                base_query, start, length
            );

            let rows = client.query(&paginated_query, &[]).await.map_err(|e| {
                eprintln!("Query error: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

            rows_to_json(&rows)
        } else {
            vec![]
        };

        Ok(PaginatedResponse { total, data })
    } else {
        Ok(PaginatedResponse {
            total: 0,
            data: vec![],
        })
    }
}

#[handler]
pub fn list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let (start, length) = parse_pagination(&pagination);

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

    let _ = get_external_pg_client(conn, ext_database_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[handler]
pub async fn query_object_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i16>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let (start, length) = parse_pagination(&pagination);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let client = get_external_pg_client(conn, ext_database_id).await?;

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

    let response = query_with_pagination(&client, query, start, length).await?;
    Ok(Json(response))
}

#[handler]
pub fn query_whitelist_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i16>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let (start, length) = parse_pagination(&pagination);

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

    let client = get_external_pg_client(conn, ext_database_id).await?;

    let mut success_response: Option<Value> = None;
    let mut results: Vec<Value> = Vec::new();

    let mut last_name: Option<String> = None;
    let mut last_action: Option<String> = None;
    let mut last_query: Option<String> = None;
    let mut last_affected = 0;

    let parts = split_manual_query(&entry_manual_ext_database.query);
    for part in parts {
        let mut affected = 0;
        let mut error: Option<String> = None;
        match extract_query_parts(&part) {
            Some((name, action)) => {
                if is_sql_type(&part, "(SELECT|WITH)") && results.len() == 0 {
                    let query = format!("{0} LIMIT 1", &part);
                    match client.query(&query, &[]).await {
                        Ok(rows) => {
                            let columns_info = extract_columns_info(&rows);
                            match diesel::insert_into(tbl_query_manual::table)
                                .values(QueryManual {
                                    id: common::generate_id(),
                                    ext_database_id,
                                    query: part.to_string(),
                                    created_by: jwt_auth.claims.username.clone(),
                                    dt_created: chrono::Utc::now().naive_utc(),
                                    updated_by: None,
                                    dt_updated: None,
                                    version: 0,
                                })
                                .get_result::<QueryManual>(conn)
                            {
                                Ok(inserted) => {
                                    success_response = Some(json!({
                                        "id": inserted.id,
                                        "header": columns_info
                                    }));
                                }
                                Err(e) => {
                                    eprintln!("Inserting error: {}", e);
                                    common::error_message(
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        "information.internalServerError",
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            results.push(json!({
                                "message": format!("{}", e)
                            }));
                        }
                    }
                } else if is_sql_type(&part, "(DROP|CREATE|ALTER)") {
                    match client.execute(&part, &[]).await {
                        Ok(_) => {
                            affected = 1;
                        }
                        Err(e) => {
                            error = Some(format!("{}", e));
                        }
                    }
                } else if is_sql_type(&part, "(INSERT|(UPDATE|DELETE)\\s.+\\s?WHERE)") {
                    match client.execute(&part, &[]).await {
                        Ok(row) => {
                            affected = row;
                        }
                        Err(e) => {
                            error = Some(format!("{}", e));
                        }
                    }
                } else if is_only_comment(&part) {
                    continue;
                } else {
                    error = Some(String::from("Abnormal"));
                }

                if let Some(err_msg) = error {
                    if let (Some(last_name), Some(last_action), Some(last_query)) =
                        (&last_name, &last_action, &last_query)
                    {
                        results.push(json!({
                            "name": last_name,
                            "action": last_action,
                            "query": last_query,
                            "affected": last_affected,
                        }));
                    }

                    results.push(json!({
                        "name": name,
                        "action": action,
                        "query": &part,
                        "message": err_msg,
                    }));

                    last_name = None;
                    last_action = None;
                    last_query = None;
                    last_affected = 0;
                } else if "select" != action {
                    if last_name.as_deref() != Some(&name)
                        || last_action.as_deref() != Some(&action)
                    {
                        if let (Some(last_name_val), Some(last_action_val), Some(last_query_val)) =
                            (&last_name, &last_action, &last_query)
                        {
                            results.push(json!({
                                "name": last_name_val,
                                "action": last_action_val,
                                "query": last_query_val,
                                "affected": last_affected,
                            }));
                        }

                        last_name = Some(name.clone());
                        last_action = Some(action.clone());
                        last_query = Some(part.clone());
                        last_affected = affected;
                    } else {
                        last_affected += affected;
                    }
                }
            }
            None => {
                println!("No match found for part: {}", part);
                results.push(json!({ "error": &part }));
            }
        }
    }

    if let (Some(last_name_val), Some(last_action_val), Some(last_query_val)) =
        (&last_name, &last_action, &last_query)
    {
        results.push(json!({
            "name": last_name_val,
            "action": last_action_val,
            "query": last_query_val,
            "affected": last_affected,
        }));
        Ok(Json(json!({ "data": results })))
    } else if !results.is_empty() {
        Ok(Json(json!({ "data": results })))
    } else if let Some(success) = success_response {
        Ok(Json(success))
    } else {
        Ok(Json(json!({ "message": "No valid query executed" })))
    }
}

#[handler]
pub async fn query_manual_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(query_manual_id): Path<i64>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    validate_id(query_manual_id)?;
    let (start, length) = parse_pagination(&pagination);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (client, query_str) = get_query_manual_client(conn, query_manual_id).await?;
    if let Some(count_query) = convert_to_count_query(&query_str) {
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
                &query_str, start, length
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

    let (rows, _) = get_query_manual_row(conn, query_manual_id).await?;
    let results = rows_to_json(&rows);
    Ok(Json(DataResponse { data: results }))
}

#[handler]
pub async fn query_manual_sql_insert(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path((query_manual_id, include_column_name_flag, number_line_per_action)): Path<(
        i64,
        i16,
        i16,
    )>,
) -> poem::Result<impl IntoResponse> {
    validate_id(query_manual_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (rows, query_str) = get_query_manual_row(conn, query_manual_id).await?;
    match extract_query_parts(&query_str) {
        Some((name, _)) => {
            let results = rows_to_insert_query_string(
                &name,
                include_column_name_flag,
                number_line_per_action,
                &rows,
            );
            Ok(Json(DataResponse { data: results }))
        }
        None => Err(common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.notFound",
        )),
    }
}

#[handler]
pub async fn query_manual_sql_update(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path((query_manual_id, multiple_line_flag, first_amount_conditioned)): Path<(i64, i16, i16)>,
) -> poem::Result<impl IntoResponse> {
    validate_id(query_manual_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (rows, query_str) = get_query_manual_row(conn, query_manual_id).await?;
    match extract_query_parts(&query_str) {
        Some((name, _)) => {
            let results = rows_to_update_query_string(
                &name,
                multiple_line_flag,
                first_amount_conditioned,
                &rows,
            );
            Ok(Json(DataResponse { data: results }))
        }
        None => Err(common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.notFound",
        )),
    }
}

#[handler]
pub async fn query_manual_csv(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path((query_manual_id, header_flag, delimiter)): Path<(i64, i16, String)>,
) -> poem::Result<impl IntoResponse> {
    validate_id(query_manual_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (rows, _) = get_query_manual_row(conn, query_manual_id).await?;
    let results = rows_to_csv_string(header_flag, &delimiter, &rows);
    Ok(Json(DataResponse { data: results }))
}

#[handler]
pub async fn query_manual_json(
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

    let (rows, _) = get_query_manual_row(conn, query_manual_id).await?;
    let results = rows_to_json_string(&rows);
    Ok(Json(DataResponse { data: results }))
}

#[handler]
pub async fn query_manual_xml(
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

    let (rows, query_str) = get_query_manual_row(conn, query_manual_id).await?;
    match extract_query_parts(&query_str) {
        Some((name, _)) => {
            let results = rows_to_xml_string(&name, &rows);
            Ok(Json(DataResponse { data: results }))
        }
        None => Err(common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.notFound",
        )),
    }
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

    let client = get_external_pg_client(conn, ext_database_id).await?;
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
    let (start, length) = parse_pagination(&pagination);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let client = get_external_pg_client(conn, ext_database_id).await?;
    let query = format!("SELECT * FROM {}", entity_name);
    let response = query_with_pagination(&client, &query, start, length).await?;
    Ok(Json(response))
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

    let (ext_database_id, query_string): (i16, String) = tbl_ext_database_query::table
        .filter(tbl_ext_database_query::id.eq(ext_database_query_id))
        .filter(tbl_ext_database_query::is_del.eq(0))
        .select((
            tbl_ext_database_query::ext_database_id,
            tbl_ext_database_query::query,
        ))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let client = get_external_pg_client(conn, ext_database_id).await?;
    let rows = client.query(&query_string, &[]).await.map_err(|e| {
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
    let (start, length) = parse_pagination(&pagination);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (ext_database_id, query_string): (i16, String) = tbl_ext_database_query::table
        .filter(tbl_ext_database_query::id.eq(ext_database_query_id))
        .filter(tbl_ext_database_query::is_del.eq(0))
        .select((
            tbl_ext_database_query::ext_database_id,
            tbl_ext_database_query::query,
        ))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let client = get_external_pg_client(conn, ext_database_id).await?;
    let response = query_with_pagination(&client, &query_string, start, length).await?;
    Ok(Json(response))
}
