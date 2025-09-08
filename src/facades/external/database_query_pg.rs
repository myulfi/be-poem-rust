use crate::models::common::{DataResponse, PaginatedResponse};
use crate::models::external::database::{EntryQueryManual, ExternalDatabaseQuery, QueryManual};
use crate::schema::tbl_ext_database;
use crate::schema::{tbl_ext_database_query, tbl_mt_database_type, tbl_query_manual};
use crate::utils::common::{self, parse_pagination, validate_id};
use crate::{db::DbPool, models::common::Pagination};
use diesel::prelude::*;
use poem::IntoResponse;
use poem::web::Query;
use poem::{
    handler,
    http::StatusCode,
    web::{Json, Path},
};
use regex::Regex;
use rust_decimal::Decimal;
use serde_json::{Map, Value, json as json_macro};
use std::fmt::Write;
use tokio_postgres::{Client, NoTls, Row};
use umya_spreadsheet::Style;
use umya_spreadsheet::structs::Fill;
use umya_spreadsheet::structs::Font;
use umya_spreadsheet::structs::PatternFill;
use umya_spreadsheet::{Color, PatternValues};
use umya_spreadsheet::{new_file, writer};

use tokio_postgres::types::Oid;

fn get_ext_database_info(
    conn: &mut PgConnection,
    ext_database_id: i64,
) -> poem::Result<(String, String)> {
    let (ip, port, username, password, db_name, mt_database_type_id): (
        String,
        i16,
        String,
        String,
        String,
        i16,
    ) = tbl_ext_database::table
        .filter(tbl_ext_database::id.eq(ext_database_id))
        .filter(tbl_ext_database::is_del.eq(0))
        .select((
            tbl_ext_database::ip,
            tbl_ext_database::port,
            tbl_ext_database::username,
            tbl_ext_database::password,
            tbl_ext_database::db_name,
            tbl_ext_database::mt_database_type_id,
        ))
        .first::<(String, i16, String, String, String, i16)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let (url, pagination): (String, String) = tbl_mt_database_type::table
        .filter(tbl_mt_database_type::id.eq(mt_database_type_id))
        .select((tbl_mt_database_type::url, tbl_mt_database_type::pagination))
        .first::<(String, String)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    // let url = format!("postgres://{0}:{1}@{2}", usr, password, db_connection)
    let url = url
        .replace("{0}", &username)
        .replace("{1}", &password)
        .replace("{2}", &ip)
        .replace("{3}", &port.to_string())
        .replace("{4}", &db_name);

    Ok((url, pagination))
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

async fn get_query_manual_client(
    conn: &mut PgConnection,
    query_manual_id: i64,
) -> poem::Result<(Client, String)> {
    let (ext_database_id, query_str): (i64, String) = tbl_query_manual::table
        .filter(tbl_query_manual::id.eq(query_manual_id))
        .select((tbl_query_manual::ext_database_id, tbl_query_manual::query))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let (client, _) = get_external_pg_client(conn, ext_database_id).await?;
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
    ext_database_id: i64,
) -> poem::Result<(Client, String)> {
    let (url, pagination) = get_ext_database_info(conn, ext_database_id)?;
    let client = connect_to_external_database(&url).await?;
    Ok((client, pagination))
}

async fn query_with_pagination(
    client: &Client,
    pagination: &str,
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
            let paginated_query = pagination
                .replace("{0}", base_query)
                .replace("{1}", &start.to_string())
                .replace("{2}", &length.to_string());

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
pub async fn connect(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i64>,
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
    Path(ext_database_id): Path<i64>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let (start, length) = parse_pagination(&pagination);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (client, pagination) = get_external_pg_client(conn, ext_database_id).await?;

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

    let response = query_with_pagination(&client, &pagination, query, start, length).await?;
    Ok(Json(response))
}

#[handler]
pub fn query_whitelist_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_database_id): Path<i64>,
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
    Path(ext_database_id): Path<i64>,
    Json(mut entry_manual_ext_database): Json<EntryQueryManual>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (client, pagination) = get_external_pg_client(conn, ext_database_id).await?;

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
                    let query = pagination
                        .replace("{0}", &part)
                        .replace("{1}", "0")
                        .replace("{2}", "1");
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
                                    success_response = Some(json_macro!({
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
                            results.push(json_macro!({
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
                        results.push(json_macro!({
                            "name": last_name,
                            "action": last_action,
                            "query": last_query,
                            "affected": last_affected,
                        }));
                    }

                    results.push(json_macro!({
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
                            results.push(json_macro!({
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
                results.push(json_macro!({ "error": &part }));
            }
        }
    }

    if let (Some(last_name_val), Some(last_action_val), Some(last_query_val)) =
        (&last_name, &last_action, &last_query)
    {
        results.push(json_macro!({
            "name": last_name_val,
            "action": last_action_val,
            "query": last_query_val,
            "affected": last_affected,
        }));
        Ok(Json(json_macro!({ "data": results })))
    } else if !results.is_empty() {
        Ok(Json(json_macro!({ "data": results })))
    } else if let Some(success) = success_response {
        Ok(Json(success))
    } else {
        Ok(Json(json_macro!({ "message": "No valid query executed" })))
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

    let (rows, _) = get_query_manual_row(conn, query_manual_id).await?;
    let results = rows_to_xlsx_bytes(first_amount_combined, &rows)?;

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
    // Ok(Json(DataResponse { data: results }))
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
    Path((ext_database_id, entity_name)): Path<(i64, String)>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (client, pagination) = get_external_pg_client(conn, ext_database_id).await?;
    let query = pagination
        .replace("{0}", &format!("SELECT * FROM {0}", entity_name))
        .replace("{1}", "0")
        .replace("{2}", "1");
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
    Path((ext_database_id, entity_name)): Path<(i64, String)>,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let (start, length) = parse_pagination(&pagination);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (client, pagination) = get_external_pg_client(conn, ext_database_id).await?;
    let query = format!("SELECT * FROM {}", entity_name);
    let response = query_with_pagination(&client, &pagination, &query, start, length).await?;
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

    let (ext_database_id, query_string): (i64, String) = tbl_ext_database_query::table
        .filter(tbl_ext_database_query::id.eq(ext_database_query_id))
        .filter(tbl_ext_database_query::is_del.eq(0))
        .select((
            tbl_ext_database_query::ext_database_id,
            tbl_ext_database_query::query,
        ))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let (client, pagination) = get_external_pg_client(conn, ext_database_id).await?;
    let query = pagination
        .replace("{0}", &query_string)
        .replace("{1}", "0")
        .replace("{2}", "1");
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

    let (ext_database_id, query_string): (i64, String) = tbl_ext_database_query::table
        .filter(tbl_ext_database_query::id.eq(ext_database_query_id))
        .filter(tbl_ext_database_query::is_del.eq(0))
        .select((
            tbl_ext_database_query::ext_database_id,
            tbl_ext_database_query::query,
        ))
        .first(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let (client, pagination) = get_external_pg_client(conn, ext_database_id).await?;
    let response =
        query_with_pagination(&client, &pagination, &query_string, start, length).await?;
    Ok(Json(response))
}

// pub fn convert_select_to_count(query: &str) -> Option<String> {
//     let query_lower = query.to_lowercase();

//     // Cari posisi "from"
//     let from_pos = query_lower.find("from")?;

//     // Split dari FROM ke akhir
//     let (_, after_select) = query.split_at(from_pos);

//     // Deteksi kalau ada ORDER, LIMIT, OFFSET dan buang
//     let mut cleaned = after_select.trim().to_string();

//     for clause in ["order by", "limit", "offset"] {
//         if let Some(pos) = cleaned.to_lowercase().find(clause) {
//             cleaned = cleaned[..pos].trim().to_string();
//         }
//     }

//     // Gabungkan ulang menjadi SELECT count(*) FROM ...
//     let count_query = format!("SELECT count(*) {}", cleaned);
//     Some(count_query)
// }

pub fn convert_to_count_query(raw_query: &str) -> Option<String> {
    let lower = raw_query.to_lowercase();

    // Cari posisi kata "from" pertama
    let from_pos = lower.find(" from ")?; // spasi penting agar tidak salah match

    // Ambil bagian FROM ke akhir
    let from_clause = &raw_query[from_pos..];

    // Buang ORDER BY, LIMIT, OFFSET jika ada
    let clauses_to_remove = [" order by ", " limit ", " offset ", " fetch "];
    let mut clean_clause = from_clause.to_string();
    for clause in clauses_to_remove {
        if let Some(pos) = clean_clause.to_lowercase().find(clause) {
            clean_clause = clean_clause[..pos].trim_end().to_string();
        }
    }

    // Susun ulang
    let result = format!("SELECT COUNT(*){}", clean_clause);
    Some(result)
}

pub fn rows_to_json(rows: &[Row]) -> Vec<Value> {
    let mut results = Vec::new();

    for row in rows {
        let mut map = Map::new();

        for (i, column) in row.columns().iter().enumerate() {
            let name = column.name();

            let value = match column.type_().name() {
                "oid" => {
                    let v: Option<Oid> = row.try_get(i).ok();
                    match v {
                        Some(oid) => Value::Number((oid as i64).into()),
                        None => Value::Null,
                    }
                }
                "int2" => {
                    let v: i16 = row.get(i);
                    Value::Number((v as i64).into())
                }
                "int4" => {
                    let v: i32 = row.get(i);
                    Value::Number((v as i64).into())
                }
                "int8" => {
                    let v: Option<i64> = row.get(i);
                    match v {
                        Some(val) => Value::Number(val.into()),
                        None => Value::Null,
                    }
                }
                "float4" | "float8" => {
                    let v: f64 = row.get(i);
                    serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                }
                "numeric" => {
                    let v: Option<rust_decimal::Decimal> = row.get(i);
                    match v {
                        Some(val) => Value::String(val.to_string()),
                        None => Value::Null,
                    }
                }
                "bool" => {
                    let v: bool = row.get(i);
                    Value::Bool(v)
                }
                "date" | "timestamp" => {
                    let val: Option<String> = row.try_get(i).ok();
                    match val {
                        Some(s) => Value::String(s),
                        None => Value::Null,
                    }
                }
                _ => {
                    let v: Option<String> = row.get(i);
                    match v {
                        Some(s) => Value::String(s),
                        None => Value::Null,
                    }
                }
            };

            map.insert(name.to_string(), value);
        }

        results.push(Value::Object(map));
    }

    results
}

pub fn rows_to_insert_query_string(
    table_name: &str,
    include_column_name_flag: i16,
    number_line_per_action: i16,
    rows: &[Row],
) -> String {
    let mut result = String::new();
    let mut batch = Vec::new();

    for (row_index, row) in rows.iter().enumerate() {
        let mut columns = Vec::new();
        let mut values = Vec::new();

        for (i, column) in row.columns().iter().enumerate() {
            let name = column.name().to_string();

            let value = match column.type_().name() {
                "oid" => {
                    let v: Option<Oid> = row.try_get(i).ok();
                    v.map(|oid| oid.to_string())
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "int2" => {
                    let v: i16 = row.get(i);
                    v.to_string()
                }
                "int4" => {
                    let v: i32 = row.get(i);
                    v.to_string()
                }
                "int8" => {
                    let v: Option<i64> = row.get(i);
                    v.map(|val| val.to_string())
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "float4" | "float8" => {
                    let v: f64 = row.get(i);
                    if v.is_finite() {
                        v.to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
                "numeric" => {
                    let v: Option<Decimal> = row.get(i);
                    v.map(|val| format!("'{}'", val.to_string()))
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "bool" => {
                    let v: bool = row.get(i);
                    v.to_string()
                }
                "date" | "timestamp" => {
                    let val: Option<String> = row.try_get(i).ok();
                    val.map(|s| format!("'{}'", s))
                        .unwrap_or_else(|| "NULL".to_string())
                }
                _ => {
                    let v: Option<String> = row.get(i);
                    v.map(|s| format!("'{}'", s.replace('\'', "''")))
                        .unwrap_or_else(|| "NULL".to_string())
                }
            };

            columns.push(name);
            values.push(value);
        }

        // Tambahkan values ke dalam batch
        batch.push(values.join(", "));

        let is_last = row_index == rows.len() - 1;
        let batch_size = number_line_per_action.max(1) as usize;

        if batch.len() == batch_size || is_last {
            if include_column_name_flag == 1 {
                let _ = write!(
                    &mut result,
                    "INSERT INTO {} ({}) VALUES ({});\n",
                    table_name,
                    columns.join(", "),
                    batch
                        .iter()
                        .map(|v| format!("({})", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            } else {
                let _ = write!(
                    &mut result,
                    "INSERT INTO {} VALUES ({});\n",
                    table_name,
                    batch
                        .iter()
                        .map(|v| format!("({})", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            batch.clear();
        }
    }

    result
}

pub fn rows_to_update_query_string(
    table_name: &str,
    multiple_line_flag: i16,
    first_amount_conditioned: i16,
    rows: &[Row],
) -> String {
    let mut result = String::new();

    for row in rows {
        let mut set_clauses = Vec::new();
        let mut where_clauses = Vec::new();
        let mut column_names = Vec::new();

        for (_, column) in row.columns().iter().enumerate() {
            column_names.push(column.name().to_string());
        }

        for (i, name) in column_names.iter().enumerate() {
            let value = match row.columns()[i].type_().name() {
                "oid" => {
                    let v: Option<Oid> = row.try_get(i).ok();
                    v.map(|oid| oid.to_string())
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "int2" => {
                    let v: i16 = row.get(i);
                    v.to_string()
                }
                "int4" => {
                    let v: i32 = row.get(i);
                    v.to_string()
                }
                "int8" => {
                    let v: Option<i64> = row.get(i);
                    v.map(|val| val.to_string())
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "float4" | "float8" => {
                    let v: f64 = row.get(i);
                    if v.is_finite() {
                        v.to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
                "numeric" => {
                    let v: Option<Decimal> = row.get(i);
                    v.map(|val| format!("'{}'", val.to_string()))
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "bool" => {
                    let v: bool = row.get(i);
                    v.to_string()
                }
                "date" | "timestamp" => {
                    let val: Option<String> = row.try_get(i).ok();
                    val.map(|s| format!("'{}'", s))
                        .unwrap_or_else(|| "NULL".to_string())
                }
                _ => {
                    let v: Option<String> = row.get(i);
                    v.map(|s| format!("'{}'", s.replace('\'', "''")))
                        .unwrap_or_else(|| "NULL".to_string())
                }
            };

            if (i as i16) < first_amount_conditioned {
                where_clauses.push(format!("{} = {}", name, value));
            } else {
                set_clauses.push(format!("{} = {}", name, value));
            }
        }

        let mut update_sql = String::new();

        if multiple_line_flag == 1 {
            let _ = writeln!(&mut update_sql, "UPDATE {}", table_name);
            if !set_clauses.is_empty() {
                let _ = writeln!(
                    &mut update_sql,
                    "SET {}",
                    set_clauses
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            if i == 0 {
                                format!("{}", v)
                            } else {
                                format!(", {}", v)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
            if !where_clauses.is_empty() {
                let _ = writeln!(&mut update_sql, "WHERE {};", where_clauses.join(" AND "));
            } else {
                update_sql.push_str(";\n");
            }

            update_sql.push('\n'); // <- Tambahkan newline antar statement
        } else {
            let _ = write!(
                &mut update_sql,
                "UPDATE {} SET {}",
                table_name,
                set_clauses.join(", ")
            );
            if !where_clauses.is_empty() {
                let _ = write!(&mut update_sql, " WHERE {};", where_clauses.join(" AND "));
            }
            update_sql.push('\n');
        }

        result.push_str(&update_sql);
    }

    result
}

pub fn rows_to_xlsx_bytes(first_amount_combined: i16, rows: &[Row]) -> poem::Result<Vec<u8>> {
    if rows.is_empty() {
        return Ok(vec![]);
    }

    // Buat workbook baru
    let mut book = new_file();
    let sheet_name = "Sheet1";
    let _ = book.new_sheet(sheet_name);

    let columns = rows[0].columns();
    let mut row_idx = 1;

    let mut header_style = Style::default();

    // Font
    let mut font = Font::default();
    font.set_bold(true);
    font.set_color({
        let mut color = Color::default();
        color.set_argb("FFFFFF");
        color
    }); // Putih
    font.set_size(14.0);
    header_style.set_font(font);

    // Fill
    let mut fill = Fill::default();
    let mut pattern = PatternFill::default();
    pattern.set_pattern_type(PatternValues::Solid);
    pattern.set_foreground_color({
        let mut color = Color::default();
        color.set_argb("000000");
        color
    });
    pattern.set_background_color({
        let mut color = Color::default();
        color.set_argb("000000");
        color
    });
    fill.set_pattern_fill(pattern);
    header_style.set_fill(fill);

    // Header
    for (col_idx, col) in columns.iter().enumerate() {
        let col_name = col.name();
        let cell = book
            .get_sheet_by_name_mut(sheet_name)
            .unwrap()
            .get_cell_mut((col_idx as u32 + 1, row_idx));
        cell.set_value(col_name);
        cell.set_style(header_style.clone()); // Apply styling
    }

    row_idx += 1;

    let mut data_matrix: Vec<Vec<String>> = vec![];

    // Ambil data sebagai matriks string
    for row in rows {
        let mut row_data = vec![];
        for (col_idx, column) in row.columns().iter().enumerate() {
            let value = match column.type_().name() {
                "oid" => row
                    .try_get::<_, Option<u32>>(col_idx)
                    .ok()
                    .flatten()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "int2" => row
                    .try_get::<_, Option<i16>>(col_idx)
                    .ok()
                    .flatten()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "int4" => row
                    .try_get::<_, Option<i32>>(col_idx)
                    .ok()
                    .flatten()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "int8" => row
                    .try_get::<_, Option<i64>>(col_idx)
                    .ok()
                    .flatten()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "float4" | "float8" => row
                    .try_get::<_, Option<f64>>(col_idx)
                    .ok()
                    .flatten()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "numeric" => row
                    .try_get::<_, Option<rust_decimal::Decimal>>(col_idx)
                    .ok()
                    .flatten()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "bool" => row
                    .try_get::<_, Option<bool>>(col_idx)
                    .ok()
                    .flatten()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "date" | "timestamp" => row
                    .try_get::<_, Option<String>>(col_idx)
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                _ => row
                    .try_get::<_, Option<String>>(col_idx)
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
            };
            row_data.push(value);
        }
        data_matrix.push(row_data);
    }

    // Tulis data ke Excel
    for (r, row_data) in data_matrix.iter().enumerate() {
        for (c, value) in row_data.iter().enumerate() {
            book.get_sheet_by_name_mut(sheet_name)
                .unwrap()
                .get_cell_mut((c as u32 + 1, row_idx + r as u32))
                .set_value(value);
        }
    }

    // Merge cell di kolom awal (first_amount_combined)
    // let total_rows = data_matrix.len();
    // for col_idx in 0..(first_amount_combined as usize).min(columns.len()) {
    //     let mut start = 0;
    //     while start < total_rows {
    //         let current_val = &data_matrix[start][col_idx];
    //         let mut end = start + 1;

    //         while end < total_rows && data_matrix[end][col_idx] == *current_val {
    //             end += 1;
    //         }

    //         if end - start > 1 {
    //             let sheet = book.get_sheet_by_name_mut(sheet_name).unwrap();
    //             let range = CellRange::new(
    //                 (col_idx as u32 + 1, row_idx + start as u32),
    //                 (col_idx as u32 + 1, row_idx + (end as u32) - 1),
    //             );
    //             sheet.add_merge(range);
    //         }

    //         start = end;
    //     }
    // }
    let mut last_seen = vec![None; first_amount_combined as usize];
    for (r, row_data) in data_matrix.iter().enumerate() {
        for col_idx in 0..(first_amount_combined as usize) {
            if Some(&row_data[col_idx]) == last_seen[col_idx].as_ref() {
                book.get_sheet_by_name_mut(sheet_name)
                    .unwrap()
                    .get_cell_mut((col_idx as u32 + 1, row_idx + r as u32))
                    .set_value("");
            } else {
                last_seen[col_idx] = Some(row_data[col_idx].clone());
            }
        }
    }

    // Tulis ke buffer
    let mut buffer = Vec::new();
    writer::xlsx::write_writer(&book, &mut buffer).map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "xlsx.writeFailed")
    })?;

    Ok(buffer)
}

pub fn rows_to_csv_string(header_flag: i16, delimiter: &str, rows: &[Row]) -> String {
    let mut result = String::new();

    if rows.is_empty() {
        return result;
    }

    // Ambil nama kolom dari baris pertama
    if header_flag == 1 {
        let header = rows[0]
            .columns()
            .iter()
            .map(|col| col.name())
            .collect::<Vec<_>>()
            .join(delimiter);
        result.push_str(&header);
        result.push('\n');
    }

    for row in rows {
        let mut values = Vec::new();

        for (i, column) in row.columns().iter().enumerate() {
            let value = match column.type_().name() {
                "oid" => {
                    let v: Option<Oid> = row.try_get(i).ok();
                    v.map(|oid| oid.to_string())
                        .unwrap_or_else(|| "".to_string())
                }
                "int2" => {
                    let v: i16 = row.get(i);
                    v.to_string()
                }
                "int4" => {
                    let v: i32 = row.get(i);
                    v.to_string()
                }
                "int8" => {
                    let v: Option<i64> = row.get(i);
                    v.map(|val| val.to_string()).unwrap_or_default()
                }
                "float4" | "float8" => {
                    let v: f64 = row.get(i);
                    if v.is_finite() {
                        v.to_string()
                    } else {
                        "".to_string()
                    }
                }
                "numeric" => {
                    let v: Option<Decimal> = row.get(i);
                    v.map(|val| val.to_string()).unwrap_or_default()
                }
                "bool" => {
                    let v: bool = row.get(i);
                    v.to_string()
                }
                "date" | "timestamp" => {
                    let val: Option<String> = row.try_get(i).ok();
                    val.unwrap_or_default()
                }
                _ => {
                    let v: Option<String> = row.get(i);
                    v.map(|s| s.replace('"', "\"\"")).unwrap_or_default()
                }
            };

            // Bungkus string dengan kutip jika mengandung koma, kutip, atau newline
            let formatted_value =
                if value.contains(delimiter) || value.contains('"') || value.contains('\n') {
                    format!("\"{}\"", value.replace('"', "\"\""))
                } else {
                    value
                };

            values.push(formatted_value);
        }

        result.push_str(&values.join(delimiter));
        result.push('\n');
    }

    result
}

pub fn rows_to_json_string(rows: &[Row]) -> String {
    let mut json_array = Vec::new();

    for row in rows {
        let mut json_object = serde_json::Map::new();

        for (i, column) in row.columns().iter().enumerate() {
            let col_name = column.name();

            let value = match column.type_().name() {
                "oid" => {
                    let v: Option<Oid> = row.try_get(i).ok();
                    v.map(|oid| json_macro!(oid)).unwrap_or(Value::Null)
                }
                "int2" => json_macro!(row.get::<_, i16>(i)),
                "int4" => json_macro!(row.get::<_, i32>(i)),
                "int8" => {
                    let v: Option<i64> = row.try_get(i).ok();
                    v.map_or_else(|| Value::Null, |v| json_macro!(v))
                }
                "float4" | "float8" => {
                    let v: f64 = row.get(i);
                    if v.is_finite() {
                        json_macro!(v)
                    } else {
                        Value::Null
                    }
                }
                "numeric" => {
                    let v: Option<Decimal> = row.try_get(i).ok();
                    v.map(|d| json_macro!(d.to_string())).unwrap_or(Value::Null)
                }
                "bool" => json_macro!(row.get::<_, bool>(i)),
                "date" | "timestamp" => {
                    let val: Option<String> = row.try_get(i).ok();
                    val.map_or_else(|| Value::Null, |val| json_macro!(val))
                }
                _ => {
                    let v: Option<String> = row.try_get(i).ok();
                    v.map_or_else(|| Value::Null, |v| json_macro!(v))
                }
            };

            json_object.insert(col_name.to_string(), value);
        }

        json_array.push(Value::Object(json_object));
    }

    serde_json::to_string_pretty(&Value::Array(json_array)).unwrap_or("[]".to_string())
}

pub fn rows_to_xml_string(table_name: &str, rows: &[Row]) -> String {
    let mut result = String::new();
    result.push_str("<List>\n");

    for row in rows {
        result.push_str(&format!("  <{}>\n", table_name));

        for (i, column) in row.columns().iter().enumerate() {
            let col_name = column.name();
            let value = match column.type_().name() {
                "oid" => {
                    let v: Option<Oid> = row.try_get(i).ok();
                    v.map(|oid| oid.to_string()).unwrap_or_default()
                }
                "int2" => row.get::<_, i16>(i).to_string(),
                "int4" => row.get::<_, i32>(i).to_string(),
                "int8" => {
                    let v: Option<i64> = row.get(i);
                    v.map_or(String::new(), |val| val.to_string())
                }
                "float4" | "float8" => {
                    let v: f64 = row.get(i);
                    if v.is_finite() {
                        v.to_string()
                    } else {
                        String::new()
                    }
                }
                "numeric" => {
                    let v: Option<Decimal> = row.get(i);
                    v.map(|val| val.to_string()).unwrap_or_default()
                }
                "bool" => row.get::<_, bool>(i).to_string(),
                "date" | "timestamp" => {
                    let val: Option<String> = row.try_get(i).ok();
                    val.unwrap_or_default()
                }
                _ => {
                    let v: Option<String> = row.get(i);
                    v.unwrap_or_default()
                }
            };

            // Escape karakter XML khusus: &, <, >, ", '
            let escaped_value = value
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&apos;");

            result.push_str(&format!(
                "    <{}>{}</{}>\n",
                col_name, escaped_value, col_name
            ));
        }

        result.push_str(&format!("  </{}>\n", table_name));
    }

    result.push_str("</List>\n");
    result
}

pub fn extract_columns_info(rows: &[Row]) -> Vec<Value> {
    let mut columns_info = Vec::new();

    if let Some(row) = rows.get(0) {
        for column in row.columns() {
            let mut map = Map::new();
            map.insert("name".to_string(), Value::String(column.name().to_string()));
            map.insert(
                "type".to_string(),
                Value::String(column.type_().name().to_string()),
            );
            columns_info.push(Value::Object(map));
        }
    }

    columns_info
}

// fn extract_columns_info(rows: &[Row]) -> Option<Vec<Value>> {
//     rows.get(0).map(|row| {
//         row.columns()
//             .iter()
//             .map(|column| {
//                 let mut map = Map::new();
//                 map.insert("name".to_string(), Value::String(column.name().to_string()));
//                 map.insert(
//                     "type".to_string(),
//                     Value::String(column.type_().name().to_string()),
//                 );
//                 Value::Object(map)
//             })
//             .collect()
//     })
// }

pub fn split_manual_query(input: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut current = String::new();

    enum State {
        Normal,
        SingleQuote,
        DoubleQuote,
        LineComment,
        BlockComment,
        BeginEndBlock,
    }

    use State::*;

    let mut state = Normal;
    let mut begin_end_level = 0;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match state {
            Normal => match c {
                '\'' => {
                    state = SingleQuote;
                    current.push(c);
                }
                '"' => {
                    state = DoubleQuote;
                    current.push(c);
                }
                '-' => {
                    if chars.peek() == Some(&'-') {
                        current.push('-');
                        current.push('-');
                        chars.next();
                        state = LineComment;
                    } else {
                        current.push(c);
                    }
                }
                '/' => {
                    if chars.peek() == Some(&'*') {
                        current.push('/');
                        current.push('*');
                        chars.next();
                        state = BlockComment;
                    } else {
                        current.push(c);
                    }
                }
                'B' | 'b' => {
                    let mut peek_str = String::from(c);
                    for _ in 0..4 {
                        if let Some(&ch) = chars.peek() {
                            peek_str.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    if peek_str.to_uppercase() == "BEGIN" {
                        begin_end_level += 1;
                        state = BeginEndBlock;
                    }

                    current.push_str(&peek_str);
                }
                ';' => {
                    if begin_end_level == 0 {
                        let trimmed = current.trim();
                        if !trimmed.is_empty() {
                            results.push(trimmed.to_string());
                        }
                        current.clear();
                    } else {
                        current.push(c);
                    }
                }
                _ => {
                    current.push(c);
                }
            },
            SingleQuote => {
                current.push(c);
                if c == '\'' {
                    state = Normal;
                } else if c == '\\' {
                    if let Some(next_c) = chars.next() {
                        current.push(next_c);
                    }
                }
            }
            DoubleQuote => {
                current.push(c);
                if c == '"' {
                    state = Normal;
                } else if c == '\\' {
                    if let Some(next_c) = chars.next() {
                        current.push(next_c);
                    }
                }
            }
            LineComment => {
                current.push(c);
                if c == '\n' {
                    state = Normal;
                }
            }
            BlockComment => {
                current.push(c);
                if c == '*' {
                    if let Some(&'/') = chars.peek() {
                        chars.next();
                        current.push('/');
                        state = Normal;
                    }
                }
            }
            BeginEndBlock => {
                current.push(c);

                if c == 'B' || c == 'b' {
                    let mut peek_str = String::from(c);
                    for _ in 0..4 {
                        if let Some(&ch) = chars.peek() {
                            peek_str.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if peek_str.to_uppercase() == "BEGIN" {
                        begin_end_level += 1;
                    }
                    current.push_str(&peek_str[1..]);
                }

                if c == 'E' || c == 'e' {
                    let mut peek_str = String::from(c);
                    for _ in 0..2 {
                        if let Some(&ch) = chars.peek() {
                            peek_str.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if peek_str.to_uppercase() == "END" && begin_end_level > 0 {
                        begin_end_level -= 1;
                        if begin_end_level == 0 {
                            state = Normal;
                        }
                    }
                    current.push_str(&peek_str[1..]);
                }
            }
        }
    }

    // Tambahkan statement terakhir jika ada
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        results.push(trimmed.to_string());
    }

    results
}

// pub fn is_sql_type(query: &str, keyword: &str) -> bool {
//     let pattern = format!(
//         r"(?si)^\s*((--.*(\r\n|\r|\n))|(\/\*[\s\S]*?\*\/))*\s*{}\b",
//         keyword
//     );
//     Regex::new(&pattern).unwrap().is_match(query)
// }

pub fn is_sql_type(query: &str, keyword: &str) -> bool {
    // Contoh: keyword = "DROP|CREATE|ALTER"
    let pattern = format!(
        r"(?si)^\s*((--[^\n]*\n?)|(/\*[\s\S]*?\*/))*\s*{}(\s|\(|$)",
        keyword
    );
    Regex::new(&pattern).unwrap().is_match(query)
}

pub fn is_only_comment(query: &str) -> bool {
    let re = Regex::new(r"(?s)^\s*((--[^\n]*\n?)|(/\*[\s\S]*?\*/))*\s*$").unwrap();
    re.is_match(query)
}

pub fn extract_query_parts(flat_query: &str) -> Option<(String, String)> {
    // let query_pattern = r"(?is)(SELECT)\s+.*?\s+FROM\s+(\S+)
    //                     |(INSERT)\s+INTO\s+(\S+)
    //                     |(UPDATE)\s+(\S+)\s+SET
    //                     |(DELETE)\s+FROM\s+(\S+)
    //                     |((?:CREATE OR REPLACE|CREATE|REPLACE|ALTER|DROP)\s+(?:FUNCTION|TABLE|VIEW|PROCEDURE))\s+(\S+)";
    let query_pattern = r"(?is)(SELECT)\s+.*?\s+FROM\s+(\S+)|(INSERT)\s+INTO\s+(\S+)|(UPDATE)\s+(\S+)\s+SET|(DELETE)\s+FROM\s+(\S+)|(CREATE|REPLACE|ALTER|DROP)\s+(\S+)";

    let re = Regex::new(query_pattern).unwrap();

    if let Some(caps) = re.captures(flat_query) {
        for i in (2..=10).rev().step_by(2) {
            if let (Some(name_match), Some(action_match)) = (caps.get(i), caps.get(i - 1)) {
                let name = name_match.as_str().to_lowercase();
                let action = action_match.as_str().to_lowercase();
                return Some((name, action));
            }
        }
    }
    None
}
