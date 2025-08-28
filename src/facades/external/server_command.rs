use std::io::{Read, Write};
use std::net::TcpStream;

use crate::db::DbPool;
use crate::models::common::DataResponse;
use crate::schema::tbl_ext_server;
use crate::utils::common;
use diesel::prelude::*;

use poem::web::Query;
use serde::{Deserialize, Serialize};
use serde_json::json as json_marco;
use ssh2::Session;

use poem::{
    IntoResponse, Result, handler,
    http::StatusCode,
    web::{Data, Json, Path},
};

use tempfile::NamedTempFile;

#[derive(Deserialize)]
struct DirectoryPagination {
    pub start: Option<i64>,
    pub length: Option<i64>,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub directory: String,
}

fn get_server_data(
    conn: &mut PgConnection,
    ext_server_id: i16,
) -> Result<(i16, String, i16, String, Option<String>, Option<String>), poem::Error> {
    tbl_ext_server::table
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
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))
}

fn create_ssh_session(
    ip: &str,
    port: i16,
    username: &str,
    password: &Option<String>,
    private_key: &Option<String>,
) -> Result<Session, poem::Error> {
    let tcp = TcpStream::connect(format!("{}:{}", ip, port))
        .map_err(|_| common::error_message(StatusCode::BAD_REQUEST, "ssh.connectionFailed"))?;

    let mut session = Session::new().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.sessionInitFailed")
    })?;

    session.set_tcp_stream(tcp);
    session.handshake().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.handshakeFailed")
    })?;

    if let Some(pwd) = password {
        session
            .userauth_password(username, pwd)
            .map_err(|_| common::error_message(StatusCode::UNAUTHORIZED, "ssh.authFailed"))?;
    } else if let Some(key) = private_key {
        let mut temp_key = NamedTempFile::new().map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.tempFileFailed")
        })?;
        temp_key.write_all(key.as_bytes()).map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.writeKeyFailed")
        })?;

        session
            .userauth_pubkey_file(username, None, &temp_key.path(), None)
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

    Ok(session)
}

fn run_ssh_command(session: &Session, command: &str) -> Result<String, poem::Error> {
    let mut channel = session.channel_session().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.channelFailed")
    })?;

    channel.exec(command).map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.commandFailed")
    })?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|_| common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.readFailed"))?;
    channel.wait_close().ok();
    Ok(output)
}

#[derive(Debug, Serialize)]
pub struct PaginatedDirectoryResponse<T> {
    pub total: i64,
    pub data: Vec<T>,
    pub directory: Vec<String>,
}

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

    let (mt_server_type_id, ip, port, username, password, private_key) =
        get_server_data(conn, ext_server_id)?;

    let session = create_ssh_session(&ip, port, &username, &password, &private_key)?;

    let command = match mt_server_type_id {
        1 => "pwd",
        2 => "cd",
        _ => {
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ssh.unknownServerType",
            ));
        }
    };

    let output = run_ssh_command(&session, command)?;
    let dir = output.lines().next().unwrap_or("").trim();

    let dir_vec: Vec<String> = match mt_server_type_id {
        1 => dir
            .split('/')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect(),
        2 => dir
            .split('\\')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect(),
        _ => vec![],
    };

    Ok(Json(DataResponse { data: dir_vec }))
}

#[handler]
pub async fn directory(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i16>,
    Query(pagination): Query<DirectoryPagination>,
) -> Result<impl IntoResponse> {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (mt_server_type_id, ip, port, username, password, private_key) =
        get_server_data(conn, ext_server_id)?;

    let session = create_ssh_session(&ip, port, &username, &password, &private_key)?;

    let mut dir_path = pagination.directory.trim().to_string();
    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    let command = match mt_server_type_id {
        1 => format!(
            r#"cd "{}" && ls -A | while read f; do [ -e "$f" ] || continue; stat --format="%n|%s|%w|%y|%U|%A|%F" "$f"; done"#,
            dir_path
        ),
        2 => format!(
            r#"powershell -Command "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8;{}""#,
            format!(
                r#"
                    $search = "{}";
                    $path = '{}';
                    $items = Get-ChildItem -LiteralPath $path | Where-Object {{
                        $search -eq "" -or $_.Name -and $_.Name.ToLower().Contains($search)
                    }};
                    $sorted = $items | Sort-Object {} {};
                    Write-Output $sorted.Count;
                    $sorted | Select-Object -Skip {} -First {} | ForEach-Object {{
                        '{{0}}|{{1}}|{{2}}|{{3}}|{{4}}|{{5}}|{{6}}' -f $_.Name, $_.Length, $_.CreationTimeUtc, $_.LastWriteTimeUtc, $_.Attributes, $_.Mode, $_.PSIsContainer
                    }}
                "#,
                pagination.search.as_deref().unwrap_or("").to_lowercase(),
                dir_path,
                match pagination.sort.as_deref() {
                    Some("createdDate") => "CreationTimeUtc",
                    _ => "Name",
                },
                match pagination.dir.as_deref() {
                    Some("desc") => "-Descending",
                    _ => "", // ascending (default), TIDAK pakai parameter
                },
                start,
                length
            ).replace('"', "\\\"").replace('\n', " ").replace('\r', "")
        ),
        _ => {
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ssh.unknownServerType",
            ));
        }
    };

    let output = run_ssh_command(&session, &command)?;
    let mut data = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 7 {
                return None;
            }

            if 1 == mt_server_type_id
                && !pagination.search.as_deref().unwrap_or("").is_empty()
                && !parts[0]
                    .to_lowercase()
                    .contains(&pagination.search.as_deref().unwrap_or("").to_lowercase())
            {
                None
            } else {
                let perms = parts[6];

                let directory_flag = match mt_server_type_id {
                    1 => {
                        if parts[6].trim() == "directory" {
                            1
                        } else {
                            0
                        }
                    }
                    2 => {
                        if parts[6].trim().to_lowercase() == "True" {
                            1
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };
                Some(json_marco!({
                    "name": parts[0],
                    "directoryFlag" : directory_flag,
                    "size": parts[1].parse::<u64>().unwrap_or(0),
                    "created_date": parts[2],
                    "modified_date": parts[3],
                    "owner": parts[4],
                    "status": {
                        "read": perms.contains("r"),
                        "write": perms.contains("w"),
                        "execute": perms.contains("x")
                    }
                }))
            }
        })
        .collect::<Vec<_>>();

    let total = match mt_server_type_id {
        1 => data.len() as i64,
        2 => output
            .lines()
            .next()
            .and_then(|first| first.trim().parse::<usize>().ok())
            .unwrap_or(0) as i64,
        _ => {
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ssh.unknownServerType",
            ));
        }
    };

    data.sort_by(|a, b| {
        let (a_val, b_val) = match pagination
            .sort
            .as_deref()
            .unwrap_or("name")
            .to_lowercase()
            .as_str()
        {
            "createddate" => (
                a.get("created_date").and_then(|v| v.as_str()),
                b.get("created_date").and_then(|v| v.as_str()),
            ),
            _ => (
                a.get("name").and_then(|v| v.as_str()),
                b.get("name").and_then(|v| v.as_str()),
            ),
        };

        match (a_val, b_val) {
            (Some(a), Some(b)) => {
                if pagination.dir.as_deref().unwrap_or("asc").to_lowercase() == "desc" {
                    b.cmp(a)
                } else {
                    a.cmp(b)
                }
            }
            _ => std::cmp::Ordering::Equal,
        }
    });

    let paginated_data = match mt_server_type_id {
        1 => data
            .into_iter()
            .skip(start as usize)
            .take(length as usize)
            .collect::<Vec<_>>(),
        2 => data,
        _ => {
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ssh.unknownServerType",
            ));
        }
    };

    let directory_parts: Vec<String> = dir_path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(Json(PaginatedDirectoryResponse {
        total: total,
        data: paginated_data,
        directory: directory_parts,
    }))
}
