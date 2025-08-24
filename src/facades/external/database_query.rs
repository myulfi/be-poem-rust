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
