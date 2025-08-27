use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;

use crate::db::DbPool;
use crate::models::common::DataResponse;
use crate::schema::tbl_ext_server;
use crate::utils::common;
use diesel::prelude::*;

use ssh2::Session;

use poem::{
    IntoResponse, Result, handler,
    http::StatusCode,
    web::{Data, Json, Path},
};

use serde_json::json;
use tempfile::NamedTempFile;

#[handler]
pub async fn connect(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i16>,
) -> Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (mt_server_type_id, ip, port, username, password, private_key) = tbl_ext_server::table
        .filter(tbl_ext_server::id.eq(ext_server_id))
        .filter(tbl_ext_server::is_del.eq(0))
        .select((
            tbl_ext_server::mt_server_type_id,
            tbl_ext_server::ip,
            tbl_ext_server::port,
            tbl_ext_server::username,
            tbl_ext_server::password,
            tbl_ext_server::private_key,
        ))
        .first::<(i16, String, i16, String, Option<String>, Option<String>)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let tcp = TcpStream::connect(format!("{}:{}", ip, port))
        .map_err(|_| common::error_message(StatusCode::BAD_REQUEST, "ssh.connectionFailed"))?;

    let mut session = Session::new().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.sessionInitFailed")
    })?;
    session.set_tcp_stream(tcp);
    session.handshake().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.handshakeFailed")
    })?;

    if let Some(ref pwd) = password {
        session
            .userauth_password(&username, pwd)
            .map_err(|_| common::error_message(StatusCode::UNAUTHORIZED, "ssh.authFailed"))?;
    } else if let Some(ref key) = private_key {
        let mut temp_key = NamedTempFile::new().map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.tempFileFailed")
        })?;

        temp_key.write_all(key.as_bytes()).map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.writeKeyFailed")
        })?;

        let path: PathBuf = temp_key.path().into();

        session
            .userauth_pubkey_file(&username, None, &path, None)
            .map_err(|_| common::error_message(StatusCode::UNAUTHORIZED, "ssh.authFailed"))?;

        // let private_key_path = Path::new("/path/to/id_rsa");
        // session
        //     .userauth_pubkey_file(username, None, private_key_path, None)
        //     .map_err(|_| common::error_message(StatusCode::UNAUTHORIZED, "ssh.authFailed"))?;
    } else {
        return Err(common::error_message(
            StatusCode::UNAUTHORIZED,
            "ssh.missingCredentials",
        ));
    }

    if !session.authenticated() {
        return Err(common::error_message(
            StatusCode::UNAUTHORIZED,
            "ssh.authFailed",
        ));
    }

    // 🧪 Jalankan command remote
    let mut channel = session.channel_session().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.channelFailed")
    })?;

    if mt_server_type_id == 1 {
        channel.exec("ls -la").map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.commandFailed")
        })?;
    } else if mt_server_type_id == 2 {
        channel.exec("dir").map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.commandFailed")
        })?;
    } else {
        return Err(common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ssh.unknownServerType",
        ));
    }

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|_| common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.readFailed"))?;

    // ✅ Cetak output baris per baris
    for line in output.lines() {
        println!("{}", line);
    }

    channel.wait_close().ok();

    // ✅ Berikan output SSH ke response
    Ok(Json(DataResponse {
        data: json!({
            "output": output,
            "extServerId": ext_server_id,
        }),
    }))
}
