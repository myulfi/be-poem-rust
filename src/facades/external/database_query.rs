use std::process::Child;

use diesel::prelude::*;
use diesel::{ExpressionMethods, PgConnection};
use poem::web::{Json, Query};
use poem::{IntoResponse, handler, http::StatusCode, web::Path};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Column, Pool, Row};
use sqlx::{MySql, Postgres};

use crate::database_pool::DatabasePool;
use crate::facades::external::server_command::start_ssh_tunnel;
use crate::models::common::{
    DataResponse, LoadedMoreResponse, PaginatedLoadedMoreResponse, PaginatedResponse, Pagination,
};
use crate::models::external::database::{EntryQueryManual, ExternalDatabaseQuery, QueryManual};
use crate::schema::{tbl_ext_database_query, tbl_query_manual};
use crate::utils::common::{encode_special_chars, parse_pagination, validate_id};
use crate::utils::database::{
    convert_to_count_query, extract_columns_info_mysql, extract_columns_info_postgres,
    extract_query_parts, is_only_comment, is_sql_type, rows_to_csv_string,
    rows_to_insert_query_string, rows_to_json_mysql, rows_to_json_postgres, rows_to_json_string,
    rows_to_update_query_string, rows_to_xlsx_bytes, rows_to_xml_string, split_manual_query,
};
use crate::{
    db::DbPool,
    schema::{tbl_ext_database, tbl_mt_database_type},
    utils::common,
};
use serde_json::{Value, json};

fn get_manual_query(conn: &mut PgConnection, query_manual_id: i64) -> poem::Result<(i64, String)> {
    tbl_query_manual::table
        .filter(tbl_query_manual::id.eq(query_manual_id))
        .select((tbl_query_manual::ext_database_id, tbl_query_manual::query))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))
}

async fn get_query_manual_pool(
    conn: &mut PgConnection,
    query_manual_id: i64,
) -> poem::Result<(DatabasePool, String, i16, String)> {
    let (ext_database_id, query_string) = get_manual_query(conn, query_manual_id)?;
    let (pool, tunnel, is_use_page, pagination) = get_external_pool(conn, ext_database_id).await?;

    if let Some(mut tunnel) = tunnel {
        let _ = tunnel.kill().ok();
    };

    Ok((pool, query_string, is_use_page, pagination))
}

async fn get_query_manual_row(
    conn: &mut PgConnection,
    query_manual_id: i64,
) -> poem::Result<(Vec<Value>, Vec<String>, String)> {
    let (ext_pool, query_str, is_use_page, _) =
        get_query_manual_pool(conn, query_manual_id).await?;

    if 1 == is_use_page {
        match &ext_pool {
            DatabasePool::Postgres(pg_pool) => {
                let rows = sqlx::query(&query_str)
                    .fetch_all(pg_pool)
                    .await
                    .map_err(|e| {
                        eprintln!("Query error: {}", e);
                        poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                    })?;

                let headers = if let Some(row) = rows.first() {
                    row.columns()
                        .iter()
                        .map(|col| col.name().to_string())
                        .collect()
                } else {
                    Vec::new()
                };

                let results = rows_to_json_postgres(&rows);

                Ok((results, headers, query_str))
            }
            DatabasePool::MySql(my_pool) => {
                let rows = sqlx::query(&query_str)
                    .fetch_all(my_pool)
                    .await
                    .map_err(|e| {
                        eprintln!("Query error: {}", e);
                        poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                    })?;

                let headers = if let Some(row) = rows.first() {
                    row.columns()
                        .iter()
                        .map(|col| col.name().to_string())
                        .collect()
                } else {
                    Vec::new()
                };

                let results = rows_to_json_mysql(&rows);

                Ok((results, headers, query_str))
            }
        }
    } else {
        Ok((Vec::new(), Vec::new(), query_str))
    }
}

async fn get_external_pool(
    conn: &mut PgConnection,
    ext_database_id: i64,
) -> poem::Result<(DatabasePool, Option<Child>, i16, String)> {
    let (ext_server_id, ip, port, username, password, db_name, mt_database_type_id, is_use_page): (
        Option<i64>,
        String,
        i16,
        String,
        String,
        String,
        i16,
        i16,
    ) = tbl_ext_database::table
        .filter(tbl_ext_database::id.eq(ext_database_id))
        .filter(tbl_ext_database::is_del.eq(0))
        .select((
            tbl_ext_database::ext_server_id,
            tbl_ext_database::ip,
            tbl_ext_database::port,
            tbl_ext_database::username,
            tbl_ext_database::password,
            tbl_ext_database::db_name,
            tbl_ext_database::mt_database_type_id,
            tbl_ext_database::is_use_page,
        ))
        .first::<(Option<i64>, String, i16, String, String, String, i16, i16)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let (url, pagination): (String, String) = tbl_mt_database_type::table
        .filter(tbl_mt_database_type::id.eq(mt_database_type_id))
        .select((tbl_mt_database_type::url, tbl_mt_database_type::pagination))
        .first::<(String, String)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let (tunnel, target_ip, target_port): (Option<Child>, String, u16) =
        if let Some(id) = ext_server_id {
            let (tunnel_process, local_port) = start_ssh_tunnel(conn, id, &ip, port)?;
            (Some(tunnel_process), "localhost".into(), local_port)
        } else {
            (None, ip.clone(), port as u16)
        };

    let url = url
        .replace("{0}", &username)
        .replace("{1}", &encode_special_chars(&password))
        // .replace("{2}", &db_connection);
        .replace("{2}", &target_ip)
        .replace("{3}", &target_port.to_string())
        .replace("{4}", &db_name);
    // println!("DB connection string : {}", url);

    let pool = match mt_database_type_id {
        1 => {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
                .map_err(|e| {
                    eprintln!("PostgreSQL connection error: {}", e);
                    poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                })?;
            DatabasePool::Postgres(pool)
        }
        2 => {
            let pool = MySqlPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
                .map_err(|e| {
                    eprintln!("MySQL connection error: {}", e);
                    poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                })?;
            DatabasePool::MySql(pool)
        }
        _ => {
            eprintln!("Unsupported database type ID: {}", mt_database_type_id);
            return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
        }
    };

    Ok((pool, tunnel, is_use_page, pagination))
}

pub async fn query_with_pagination(
    conn: &mut PgConnection,
    ext_database_id: i64,
    query: &str,
    start: i64,
    length: i64,
) -> poem::Result<PaginatedLoadedMoreResponse<Value>> {
    let (ext_pool, tunnel, is_use_page, pagination) =
        get_external_pool(conn, ext_database_id).await?;
    let respone = match ext_pool {
        DatabasePool::Postgres(ref pg_pool) => {
            query_with_pagination_postgres(pg_pool, &pagination, query, is_use_page, start, length)
                .await
        }
        DatabasePool::MySql(ref my_pool) => {
            query_with_pagination_mysql(my_pool, &pagination, query, is_use_page, start, length)
                .await
        }
    };

    if let Some(mut tunnel) = tunnel {
        let _ = tunnel.kill().ok();
    };

    respone
}

async fn query_with_pagination_postgres(
    pool: &Pool<Postgres>,
    pagination: &str,
    base_query: &str,
    is_use_page: i16,
    start: i64,
    length: i64,
) -> poem::Result<PaginatedLoadedMoreResponse<Value>> {
    let total = if is_use_page == 1 {
        if let Some(count_query) = convert_to_count_query(base_query) {
            sqlx::query_scalar::<_, i64>(&count_query)
                .fetch_one(pool)
                .await
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    let mut data = if total > 0 || is_use_page != 1 {
        let limit = if is_use_page == 1 { length } else { length + 1 };
        let paginated_query = pagination
            .replace("{0}", base_query)
            .replace("{1}", &start.to_string())
            .replace("{2}", &limit.to_string());

        let rows = sqlx::query(&paginated_query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                eprintln!("Query error: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        rows_to_json_postgres(&rows)
    } else {
        vec![]
    };

    if is_use_page == 1 {
        Ok(PaginatedLoadedMoreResponse::Paginated(PaginatedResponse {
            total,
            data,
        }))
    } else {
        let mut loaded = 0;
        if data.len() > length as usize {
            data.pop();
            loaded = 1;
        }
        Ok(PaginatedLoadedMoreResponse::LoadedMore(
            LoadedMoreResponse { loaded, data },
        ))
    }
}

async fn query_with_pagination_mysql(
    pool: &Pool<MySql>,
    pagination: &str,
    base_query: &str,
    is_use_page: i16,
    start: i64,
    length: i64,
) -> poem::Result<PaginatedLoadedMoreResponse<Value>> {
    let total = if is_use_page == 1 {
        if let Some(count_query) = convert_to_count_query(base_query) {
            sqlx::query_scalar::<_, i64>(&count_query)
                .fetch_one(pool)
                .await
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    let mut data = if total > 0 || is_use_page != 1 {
        let limit = if is_use_page == 1 { length } else { length + 1 };
        let paginated_query = pagination
            .replace("{0}", base_query)
            .replace("{1}", &start.to_string())
            .replace("{2}", &limit.to_string());

        let rows = sqlx::query(&paginated_query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                eprintln!("Query error: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        rows_to_json_mysql(&rows)
    } else {
        vec![]
    };

    if is_use_page == 1 {
        Ok(PaginatedLoadedMoreResponse::Paginated(PaginatedResponse {
            total,
            data,
        }))
    } else {
        let mut loaded = 0;
        if data.len() > length as usize {
            data.pop();
            loaded = 1;
        }
        Ok(PaginatedLoadedMoreResponse::LoadedMore(
            LoadedMoreResponse { loaded, data },
        ))
    }
}

async fn run_and_extract_columns(
    conn: &mut PgConnection,
    ext_database_id: i64,
    raw_query: &str,
) -> poem::Result<Vec<serde_json::Value>> {
    let (ext_pool, tunnel, _, pagination) = get_external_pool(conn, ext_database_id).await?;

    let query = pagination
        .replace("{0}", raw_query)
        .replace("{1}", "0")
        .replace("{2}", "1");

    let columns_info = match &ext_pool {
        DatabasePool::Postgres(_) => {
            let rows = ext_pool
                .fetch_all_postgres(&query)
                .await
                .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
            extract_columns_info_postgres(&rows)
        }
        DatabasePool::MySql(_) => {
            let rows = ext_pool
                .fetch_all_mysql(&query)
                .await
                .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
            extract_columns_info_mysql(&rows)
        }
    };

    if let Some(mut tunnel) = tunnel {
        let _ = tunnel.kill().ok();
    };

    Ok(columns_info)
}

fn get_whitelist_query(conn: &mut PgConnection, query_id: i64) -> poem::Result<(i64, String)> {
    tbl_ext_database_query::table
        .filter(tbl_ext_database_query::id.eq(query_id))
        .filter(tbl_ext_database_query::is_del.eq(0))
        .select((
            tbl_ext_database_query::ext_database_id,
            tbl_ext_database_query::query,
        ))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))
}

#[handler]
pub async fn connect(
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

    let (_, mut tunnel, is_use_page, _) = get_external_pool(conn, ext_database_id).await?;

    if let Some(mut tunnel) = tunnel {
        let _ = tunnel.kill().ok();
    };

    Ok(Json(DataResponse {
        data: json!({ "usePageFlag": is_use_page}),
    }))
}

#[handler]
pub async fn query_object_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i64>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_id)?;

    let (start, length) = parse_pagination(&pagination);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (ext_pool, mut tunnel, _, pagination) = get_external_pool(conn, ext_database_id).await?;

    let response = match ext_pool {
        DatabasePool::Postgres(ref pg_pool) => {
            let query = r#"
            SELECT
                objects.object_id,
                objects.object_name,
                objects.object_type
            FROM (
                SELECT
                    pg_class.oid::INT4 AS object_id,
                    views.viewname AS object_name,
                    'view' AS object_type
                FROM pg_catalog.pg_views views
                LEFT JOIN pg_class ON pg_class.relname = views.viewname
                WHERE views.schemaname = 'public'
    
                UNION ALL
    
                SELECT
                    pg_class.oid::INT4 AS object_id,
                    tables.tablename AS object_name,
                    'table' AS object_type
                FROM pg_catalog.pg_tables tables
                LEFT JOIN pg_class ON pg_class.relname = tables.tablename
                WHERE tables.schemaname = 'public'
    
                UNION ALL
    
                SELECT
                    --functions.pronamespace AS object_id,
                    functions.oid::INT4 AS object_id,
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
            query_with_pagination_postgres(pg_pool, &pagination, query, 1, start, length).await?
        }
        DatabasePool::MySql(ref my_pool) => {
            let query = r#"
            SELECT
                object_id,
                object_name,
                object_type
            FROM (
                SELECT
                    0 AS object_id,
                    TABLE_NAME AS object_name,
                    CASE WHEN TABLE_TYPE = 'BASE TABLE' THEN 'table' ELSE 'view' END AS object_type
                FROM information_schema.tables
                WHERE TABLE_SCHEMA = DATABASE()

                UNION ALL

                SELECT
                    0 AS object_id,
                    ROUTINE_NAME AS object_name,
                    ROUTINE_TYPE AS object_type
                FROM information_schema.routines
                WHERE ROUTINE_SCHEMA = DATABASE()
            ) AS objects
            WHERE 1 = 1
        "#;
            query_with_pagination_mysql(my_pool, &pagination, query, 1, start, length).await?
        }
    };

    if let Some(mut tunnel) = tunnel {
        let _ = tunnel.kill().ok();
    };

    Ok(Json(response))
}

#[handler]
pub fn query_whitelist_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i64>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_id)?;

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
    Path(ext_database_id): Path<i64>,
    Json(entry_manual_ext_database): Json<EntryQueryManual>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (ext_pool, mut tunnel, _, pagination) = get_external_pool(conn, ext_database_id).await?;

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
                if is_sql_type(&part, "(SELECT|WITH)") && results.is_empty() {
                    let query = pagination
                        .replace("{0}", &part)
                        .replace("{1}", "0")
                        .replace("{2}", "1");

                    let columns_info = match &ext_pool {
                        DatabasePool::Postgres(_) => {
                            match ext_pool.fetch_all_postgres(&query).await {
                                Ok(rows) => extract_columns_info_postgres(&rows),
                                Err(e) => {
                                    results.push(json!({ "message": format!("{}", e) }));
                                    continue;
                                }
                            }
                        }
                        DatabasePool::MySql(_) => match ext_pool.fetch_all_mysql(&query).await {
                            Ok(rows) => extract_columns_info_mysql(&rows),
                            Err(e) => {
                                results.push(json!({ "message": format!("{}", e) }));
                                continue;
                            }
                        },
                    };

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
                } else if is_sql_type(&part, "(DROP|CREATE|ALTER)") {
                    match ext_pool.execute(&part).await {
                        Ok(_) => affected = 1,
                        Err(e) => error = Some(format!("{}", e)),
                    }
                } else if is_sql_type(&part, "(INSERT|(UPDATE|DELETE)\\s.+\\s?WHERE)") {
                    match ext_pool.execute(&part).await {
                        Ok(rows) => affected = rows,
                        Err(e) => error = Some(format!("{}", e)),
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
                } else if action.to_lowercase() != "select" {
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

    if let Some(mut tunnel) = tunnel {
        let _ = tunnel.kill().ok();
    };

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

    let (ext_database_id, query_string) = get_manual_query(conn, query_manual_id)?;
    let response =
        query_with_pagination(conn, ext_database_id, &query_string, start, length).await?;
    Ok(Json(response))
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

    let (results, _, _) = get_query_manual_row(conn, query_manual_id).await?;
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

    let (rows, headers, query_str) = get_query_manual_row(conn, query_manual_id).await?;
    match extract_query_parts(&query_str) {
        Some((name, _)) => {
            let results = rows_to_insert_query_string(
                &name,
                include_column_name_flag,
                number_line_per_action,
                rows,
                headers,
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

    let (rows, headers, query_str) = get_query_manual_row(conn, query_manual_id).await?;
    match extract_query_parts(&query_str) {
        Some((name, _)) => {
            let results = rows_to_update_query_string(
                &name,
                multiple_line_flag,
                first_amount_conditioned,
                rows,
                headers,
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
pub async fn query_manual_xlsx(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path((query_manual_id, first_amount_combined)): Path<(i64, i16)>,
) -> poem::Result<impl IntoResponse> {
    validate_id(query_manual_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (rows, headers, _) = get_query_manual_row(conn, query_manual_id).await?;
    let results = rows_to_xlsx_bytes(first_amount_combined, rows, headers)?;

    Ok(poem::Response::builder()
        .status(StatusCode::OK)
        .header(
            "Content-Type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            "Content-Disposition",
            "attachment; filename=\"export.xlsx\"",
        )
        .body(results))
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

    let (rows, headers, _) = get_query_manual_row(conn, query_manual_id).await?;
    let results = rows_to_csv_string(header_flag, &delimiter, rows, headers);
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

    let (rows, headers, _) = get_query_manual_row(conn, query_manual_id).await?;
    let results = rows_to_json_string(rows, headers);
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

    let (rows, headers, query_str) = get_query_manual_row(conn, query_manual_id).await?;
    match extract_query_parts(&query_str) {
        Some((name, _)) => {
            let results = rows_to_xml_string(&name, rows, headers);
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
    Path((ext_database_id, entity_name)): Path<(i64, String)>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let query = &format!("SELECT * FROM {0}", entity_name);
    let columns_info = run_and_extract_columns(conn, ext_database_id, &query).await?;

    Ok(Json(DataResponse {
        data: Value::Array(columns_info),
    }))
}

#[handler]
pub async fn query_exact_object_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path((ext_database_id, entity_name)): Path<(i64, String)>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_id)?;

    let (start, length) = parse_pagination(&pagination);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let query = format!("SELECT * FROM {}", entity_name);
    let response = query_with_pagination(conn, ext_database_id, &query, start, length).await?;
    Ok(Json(response))
}

#[handler]
pub async fn query_exact_whitelist_run(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_query_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_database_query_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (ext_database_id, query_string) = get_whitelist_query(conn, ext_database_query_id)?;
    let columns_info = run_and_extract_columns(conn, ext_database_id, &query_string).await?;
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

    let (ext_database_id, query_string) = get_whitelist_query(conn, ext_database_query_id)?;
    let response =
        query_with_pagination(conn, ext_database_id, &query_string, start, length).await?;
    Ok(Json(response))
}
