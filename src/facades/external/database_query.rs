// async fn get_external_client(// conn: &mut PgConnection,
//     // ext_database_id: i16,
// ) -> poem::Result<(Client, String)> {
//     // Step 1: Bind listener ke port acak
//     let listener = TcpListener::bind("127.0.0.1:0").await?;
//     let local_addr = listener.local_addr()?;
//     let local_port = local_addr.port();
//     drop(listener); // Kita tidak perlu listener-nya, hanya port-nya

//     println!("Menggunakan local port: {}", local_port);

//     // Step 2: Buat SSH session
//     let ssh = Session::connect("user@remote-host", openssh::KnownHosts::Strict).await?;

//     // Step 3: Forward port remote ke port lokal yang dipilih
//     let _tunnel = ssh
//         .forward_remote_port(local_port, "127.0.0.1:3306")
//         .await?;

//     // Step 4: Koneksi ke DB via sqlx

//     let db_url = "mysql://root:%40Master87%23%21123@103.118.99.182:3306/master";
//     let pool = PgPool::connect(&db_url).await?;

//     let row: (i64,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await?;

//     println!("Hasil query: {}", row.0);
// }

// #[handler]
// pub async fn connect(
//     pool: poem::web::Data<&DbPool>,
//     _: crate::auth::middleware::JwtAuth,
//     Path(ext_database_id): Path<i16>,
// ) -> poem::Result<impl IntoResponse> {
//     let conn = &mut pool.get().map_err(|_| {
//         common::error_message(
//             StatusCode::INTERNAL_SERVER_ERROR,
//             "information.connectionFailed",
//         )
//     })?;

//     let _ = get_external_client().await?;
//     Ok(StatusCode::NO_CONTENT)
// }

use diesel::prelude::*;
use diesel::{ExpressionMethods, PgConnection};
use poem::web::{Json, Query};
use poem::{IntoResponse, handler, http::StatusCode, web::Path};
use sqlx::{Any, AnyPool, Pool, any::AnyPoolOptions};

use crate::models::common::{PaginatedResponse, Pagination};
use crate::utils::common::parse_pagination;
use crate::utils::database::{convert_to_count_query, rows_to_json};
use crate::{
    db::DbPool,
    schema::{tbl_ext_database, tbl_mt_database_type},
    utils::common,
};
use serde_json::Value;

fn get_ext_database_info(
    conn: &mut PgConnection,
    ext_database_id: i16,
) -> poem::Result<(String, String)> {
    let (username, password, db_connection, mt_database_type_id): (String, String, String, i16) =
        tbl_ext_database::table
            .filter(tbl_ext_database::id.eq(ext_database_id))
            .filter(tbl_ext_database::is_del.eq(0))
            .select((
                tbl_ext_database::username,
                tbl_ext_database::password,
                tbl_ext_database::db_connection,
                tbl_ext_database::mt_database_type_id,
            ))
            .first::<(String, String, String, i16)>(conn)
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
        .replace("{2}", &db_connection);

    Ok((url, pagination))
}

async fn connect_to_external_database(connection_str: &str) -> poem::Result<AnyPool> {
    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(connection_str)
        .await
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;
    println!("Connected to: {:?}", pool.any_kind());
    Ok(pool)
}

async fn get_external_pool(
    conn: &mut PgConnection,
    ext_database_id: i16,
) -> poem::Result<(Pool<Any>, String)> {
    let (url, pagination) = get_ext_database_info(conn, ext_database_id)?;
    let pool = connect_to_external_database(&url).await?;
    Ok((pool, pagination))
}

async fn query_with_pagination(
    pool: &Pool<Any>,
    pagination: &str,
    base_query: &str,
    start: i64,
    length: i64,
) -> poem::Result<PaginatedResponse<Value>> {
    if let Some(count_query) = convert_to_count_query(base_query) {
        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(pool)
            .await
            .map_err(|_| {
                common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "information.notFound")
            })?;

        let data = if total > 0 {
            let paginated_query = pagination
                .replace("{0}", base_query)
                .replace("{1}", &start.to_string())
                .replace("{2}", &length.to_string());

            let rows = sqlx::query(&paginated_query)
                .fetch_all(pool)
                .await
                .map_err(|e| {
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
    Path(ext_database_id): Path<i16>,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let _ = get_external_pool(conn, ext_database_id).await?;
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

    let (pool, pagination) = get_external_pool(conn, ext_database_id).await?;

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

    let response = query_with_pagination(&pool, &pagination, query, start, length).await?;
    Ok(Json(response))
}
