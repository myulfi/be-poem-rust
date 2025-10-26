use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use crate::db::DbPool;
use crate::models::common::DataResponse;
use crate::models::external::server::{
    EntryExternalServerDirectory, EntryExternalServerFile, MultipleExternalServerEntity,
    MultipleExternalServerFile,
};
use crate::schema::tbl_ext_server;
use crate::utils::common::{
    self, generate_copy_name, is_valid_directory_path, is_valid_filename, validate_id,
};
use base64::{Engine, engine::general_purpose};
use chrono::Utc;
use diesel::prelude::*;

use poem::web::Multipart;
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
use tokio::io::AsyncRead;
use tokio::io::{self};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::bytes::Bytes;
use tokio_util::io::StreamReader;

#[derive(Deserialize)]
struct DirectoryPagination {
    pub start: Option<i64>,
    pub length: Option<i64>,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub directory: String,
}

#[derive(Deserialize)]
struct FileData {
    pub name: String,
    pub directory: String,
}

fn create_ssh_session(
    conn: &mut PgConnection,
    ext_server_id: i64,
) -> Result<(Session, i16), poem::Error> {
    let (mt_server_type_id, ip, port, username, password, private_key): (
        i16,
        String,
        i16,
        String,
        Option<String>,
        Option<String>,
    ) = tbl_ext_server::table
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

    if let Some(pwd) = password {
        session
            .userauth_password(&username, &pwd)
            .map_err(|_| common::error_message(StatusCode::UNAUTHORIZED, "ssh.authFailed"))?;
    } else if let Some(key) = private_key {
        let mut temp_key = NamedTempFile::new().map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.tempFileFailed")
        })?;
        temp_key.write_all(key.as_bytes()).map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.writeKeyFailed")
        })?;

        session
            .userauth_pubkey_file(&username, None, &temp_key.path(), None)
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

    Ok((session, mt_server_type_id))
}

pub fn start_ssh_tunnel(
    conn: &mut PgConnection,
    ext_server_id: i64,
    remote_host: &str,
    remote_port: i16,
) -> Result<(Child, u16), poem::Error> {
    let (ip, port, username, password, private_key): (
        String,
        i16,
        String,
        Option<String>,
        Option<String>,
    ) = tbl_ext_server::table
        .filter(tbl_ext_server::id.eq(ext_server_id))
        .filter(tbl_ext_server::is_del.eq(0))
        .select((
            tbl_ext_server::ip,
            tbl_ext_server::port,
            tbl_ext_server::username,
            tbl_ext_server::password,
            tbl_ext_server::private_key,
        ))
        .first::<(String, i16, String, Option<String>, Option<String>)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| {
        eprintln!("Failed to bind local port: {}", e);
        poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let local_port = listener
        .local_addr()
        .map_err(|e| {
            eprintln!("Failed to get local address: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?
        .port();
    drop(listener);

    let mut temp_file = NamedTempFile::new().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.tempFileFailed")
    })?;

    let child: Child;
    if let Some(pwd) = password {
        temp_file.write_all(pwd.as_bytes()).map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.writeKeyFailed")
        })?;

        child = Command::new("wsl")
            .args([
                "sshpass",
                //"-f",
                //path_str.as_str(),
                "-p",
                &pwd,
                "ssh",
                "-o",
                "StrictHostKeyChecking=no",
                "-N",
                "-L",
                &format!("{}:{}:{}", local_port, remote_host, remote_port),
                "-p",
                &port.to_string(),
                &format!("{}@{}", username, ip),
            ])
            .spawn()
            .map_err(|e| {
                eprintln!("Failed to start SSH tunnel (password): {}", e);
                common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.tunnelFailed")
            })?;
    } else if let Some(key) = private_key {
        temp_file.write_all(key.as_bytes()).map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.writeKeyFailed")
        })?;

        child = Command::new("ssh")
            .args([
                "-N",
                "-L",
                &format!("{}:{}:{}", local_port, remote_host, remote_port),
                "-p",
                &port.to_string(),
                "-i",
                temp_file.path().to_str().ok_or_else(|| {
                    common::error_message(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ssh.invalidCredentialPath",
                    )
                })?,
                &format!("{}@{}", username, ip),
            ])
            .spawn()
            .map_err(|e| {
                eprintln!("Failed to start SSH tunnel: {}", e);
                common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.tunnelFailed")
            })?;
    } else {
        return Err(common::error_message(
            StatusCode::UNAUTHORIZED,
            "ssh.missingCredentials",
        ));
    };

    thread::sleep(Duration::from_millis(2000));
    Ok((child, local_port))
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

pub fn run_ssh_stream_command(
    session: &Session,
    command: &str,
) -> io::Result<impl AsyncRead + Unpin + Send + 'static> {
    // Setup SSH channel and exec command
    let mut channel = session
        .channel_session()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    channel
        .exec(command)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Get blocking stdout reader
    let mut stdout = channel.stream(0);

    // Channel for async streaming
    let (tx, rx) = mpsc::channel::<Result<Bytes, io::Error>>(16);

    // Spawn blocking thread to read from stdout
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match stdout.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // Send bytes to async channel
                    if tx
                        .blocking_send(Ok(Bytes::copy_from_slice(&buf[..n])))
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx.blocking_send(Err(e));
                    break;
                }
            }
        }
    });

    // Wrap ReceiverStream as AsyncRead
    let stream = ReceiverStream::new(rx);
    let async_reader = StreamReader::new(stream);

    Ok(async_reader)
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
    Path(ext_server_id): Path<i64>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;
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
pub async fn directory_list(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Query(pagination): Query<DirectoryPagination>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    let mut dir_path = pagination.directory.trim().to_string();
    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    let command = match mt_server_type_id {
        1 => format!(r#"[ -d "{}" ] && echo "1" || echo "0""#, dir_path),
        2 => format!(r#"IF EXIST "{}\" (echo 1) ELSE (echo 0)"#, dir_path),
        _ => {
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ssh.unknownServerType",
            ));
        }
    };
    let output = run_ssh_command(&session, &command)?;
    if 1 == output
        .lines()
        .next()
        .and_then(|first| first.trim().parse::<usize>().ok())
        .unwrap_or(0) as i64
    {
        let command = match mt_server_type_id {
            1 => format!(
                r#"cd "{}" && ls -A | while read f; do [ -e "$f" ] || continue; stat --format="%n|%s|%w|%y|%F|%A|%U" "$f"; done"#,
                dir_path
            ),
            2 => format!(
                r#"powershell -Command "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8;{}""#,
                if dir_path.len() > 0 {
                    format!(
                        r#"
                            $search = "{}";
                            $path = '{}/';
                            $items = Get-ChildItem -LiteralPath $path | Where-Object {{
                                $search -eq "" -or $_.Name -and $_.Name.ToLower().Contains($search)
                            }};
                            $sorted = $items | Sort-Object {} {};
                            Write-Output $sorted.Count;
                            $sorted | Select-Object -Skip {} -First {} | ForEach-Object {{
                                '{{0}}|{{1}}|{{2}}|{{3}}|{{4}}|{{5}}|{{6}}' -f $_.Name, $_.Length, $_.CreationTimeUtc, $_.LastWriteTimeUtc, $_.PSIsContainer, $_.Mode, $_.PSIsContainer
                            }}
                        "#,
                        pagination.search.as_deref().unwrap_or("").to_lowercase(),
                        dir_path,
                        match pagination.sort.as_deref() {
                            Some("modified_date") => "LastWriteTimeUtc",
                            Some("size") => "Length",
                            _ => "Name",
                        },
                        match pagination.dir.as_deref() {
                            Some("desc") => "-Descending",
                            _ => "",
                        },
                        start,
                        length
                    )
                } else {
                    format!(
                        r#"
                            $drives = [System.IO.DriveInfo]::GetDrives() | Where-Object {{ $_.IsReady }};
                            Write-Output $drives.Count;
                            foreach ($drive in $drives) {{
                                '{{0}}|{{1}}|{{2}}|{{3}}|True|d-----|-----' -f $drive.Name.TrimEnd('\'), $drive.TotalSize, $drive.RootDirectory.CreationTimeUtc, $drive.RootDirectory.LastWriteTimeUtc;
                            }}
                        "#
                    )
                }
                .replace('"', "\\\"").replace('\n', " ").replace('\r', "")
            ),
            _ => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ssh.unknownServerType",
                ));
            }
        };

        let paginated_data;
        let total;
        let output = run_ssh_command(&session, &command)?;
        if output.len() > 0 {
            let mut data = output
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();

                    if parts.len() < 6 {
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
                        let directory_flag = match mt_server_type_id {
                            1 => {
                                if parts[4].trim() == "directory" {
                                    1
                                } else {
                                    0
                                }
                            }
                            2 => {
                                if parts[4].trim().to_lowercase() == "true" {
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
                            "owner": parts[6],
                            "status": parts[5]
                        }))
                    }
                })
                .collect::<Vec<_>>();

            total = match mt_server_type_id {
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
                let sort_field = pagination.sort.as_deref().unwrap_or("name").to_lowercase();
                let is_desc = pagination
                    .dir
                    .as_deref()
                    .unwrap_or("asc")
                    .eq_ignore_ascii_case("desc");

                let ordering = match sort_field.as_str() {
                    "size" => {
                        let a_val = a.get("size").and_then(|v| v.as_u64());
                        let b_val = b.get("size").and_then(|v| v.as_u64());
                        match (a_val, b_val) {
                            (Some(a), Some(b)) => a.cmp(&b),
                            _ => std::cmp::Ordering::Equal,
                        }
                    }
                    "modified_date" => {
                        let a_val = a.get("modified_date").and_then(|v| v.as_str());
                        let b_val = b.get("modified_date").and_then(|v| v.as_str());
                        match (a_val, b_val) {
                            (Some(a), Some(b)) => a.cmp(b),
                            _ => std::cmp::Ordering::Equal,
                        }
                    }
                    _ => {
                        let a_val = a.get("name").and_then(|v| v.as_str());
                        let b_val = b.get("name").and_then(|v| v.as_str());
                        match (a_val, b_val) {
                            (Some(a), Some(b)) => a.cmp(b),
                            _ => std::cmp::Ordering::Equal,
                        }
                    }
                };

                if is_desc {
                    ordering.reverse()
                } else {
                    ordering
                }
            });

            paginated_data = match mt_server_type_id {
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
        } else {
            total = 0;
            paginated_data = vec![];
        }

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
    } else {
        return Err(common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ssh.unknownPath",
        ));
    }
}

fn check_entity_exists(
    session: &ssh2::Session,
    mt_server_type_id: i16,
    dir_path: &str,
) -> Result<bool> {
    let command = match mt_server_type_id {
        1 => format!(r#"[ -e "{}" ] && echo "1" || echo "0""#, dir_path),
        2 => format!(r#"IF EXIST "{}" (echo 1) ELSE (echo 0)"#, dir_path),
        _ => {
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ssh.unknownServerType",
            ));
        }
    };

    let output = run_ssh_command(session, &command)?;

    let exists = output
        .lines()
        .next()
        .and_then(|first| first.trim().parse::<usize>().ok())
        .unwrap_or(0)
        == 1;

    Ok(exists)
}

fn check_is_directory(
    session: &Session,
    mt_server_type_id: i16,
    path: &str,
) -> Result<bool, poem::Error> {
    let command = match mt_server_type_id {
        1 => format!(r#"[ -d "{}" ] && echo "true" || echo "false""#, path),
        2 => format!(
            r#"powershell -Command "(Test-Path '{}' -PathType Container)""#,
            path
        ),
        _ => {
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ssh.unknownServerType",
            ));
        }
    };

    let output = run_ssh_command(session, &command)?;
    Ok(output.trim().eq_ignore_ascii_case("true"))
}

#[handler]
pub async fn add_folder(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Json(entry_ext_server_directory): Json<EntryExternalServerDirectory>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    let mut dir_path = entry_ext_server_directory.dir.join("/");
    if !is_valid_directory_path(&dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    let entity_exists = check_entity_exists(
        &session,
        mt_server_type_id,
        &format!("{}/{}", dir_path, entry_ext_server_directory.nm),
    )?;

    if !entity_exists {
        let command = format!(r#"mkdir "{}/{}""#, dir_path, entry_ext_server_directory.nm);
        run_ssh_command(&session, &command)?;
        Ok(StatusCode::CREATED)
    } else {
        Err(common::error_message(
            StatusCode::CONFLICT,
            "ssh.alreadyExists",
        ))
    }
}

#[handler]
pub async fn rename_entity(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Json(entry_ext_server_directory): Json<EntryExternalServerDirectory>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    let mut dir_path = entry_ext_server_directory.dir.join("/");
    if !is_valid_directory_path(&dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    let entity_exists = check_entity_exists(
        &session,
        mt_server_type_id,
        &format!("{}/{}", dir_path, entry_ext_server_directory.old_nm),
    )?;

    if entity_exists && entry_ext_server_directory.old_nm.len() > 0 {
        let entity_exists = check_entity_exists(
            &session,
            mt_server_type_id,
            &format!("{}/{}", dir_path, entry_ext_server_directory.nm),
        )?;

        if !entity_exists {
            let command = match mt_server_type_id {
                1 => format!(
                    "mv \"{}\" \"{}\"",
                    &format!("{}/{}", dir_path, entry_ext_server_directory.old_nm),
                    &format!("{}/{}", dir_path, entry_ext_server_directory.nm)
                ),
                2 => format!(
                    "powershell -Command \"Rename-Item -Path '{}' -NewName '{}'\"",
                    &format!("{}/{}", dir_path, entry_ext_server_directory.old_nm),
                    &format!("{}/{}", dir_path, entry_ext_server_directory.nm)
                ),
                _ => {
                    return Err(common::error_message(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ssh.unknownServerType",
                    ));
                }
            };

            run_ssh_command(&session, &command)?;
            Ok(StatusCode::NO_CONTENT)
        } else {
            Err(common::error_message(
                StatusCode::CONFLICT,
                "ssh.fileAlreadyExist",
            ))
        }
    } else {
        Err(poem::Error::from_status(StatusCode::NOT_FOUND))
    }
}

#[handler]
pub async fn get_file(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Query(file_data): Query<FileData>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    if !is_valid_filename(&file_data.name) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidFilename",
        ));
    }

    let mut dir_path = file_data.directory;
    if !is_valid_directory_path(&dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    let entity_exists = check_entity_exists(
        &session,
        mt_server_type_id,
        &format!("{}/{}", dir_path, file_data.name),
    )?;

    if entity_exists {
        let command = match mt_server_type_id {
            1 => format!(r#"cat "{}/{}""#, dir_path, file_data.name),
            2 => format!(
                r#"powershell -Command "Get-Content -Path '{}\{}'"#,
                dir_path, file_data.name
            ),
            _ => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ssh.unknownServerType",
                ));
            }
        };

        let output = run_ssh_command(&session, &command)?;
        Ok(Json(json_marco!({
            "content": output
        })))
    } else {
        Err(poem::Error::from_status(StatusCode::NOT_FOUND))
    }
}

#[handler]
pub async fn download_entity(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Query(file_data): Query<FileData>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    use poem::{Body, Response};
    use tokio_util::io::ReaderStream;

    let name_array: Vec<String> = file_data
        .name
        .split("||")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if name_array.is_empty() || name_array.iter().any(|n| !is_valid_filename(n)) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidFilename",
        ));
    }

    let mut dir_path = file_data.directory;
    if !is_valid_directory_path(&dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    for name in &name_array {
        let path = format!("{}/{}", dir_path, name);
        if !check_entity_exists(&session, mt_server_type_id, &path)? {
            return Err(common::error_message(
                StatusCode::NOT_FOUND,
                &format!("ssh.entityNotFound: {}", name),
            ));
        }
    }

    let zip_flag = name_array.len() > 1
        || check_is_directory(
            &session,
            mt_server_type_id,
            &format!("{}/{}", dir_path, &name_array[0]),
        )?;

    let name = if zip_flag {
        &format!(
            "{}.{}",
            if name_array.len() == 1 {
                &name_array[0]
            } else {
                std::path::Path::new(&dir_path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            },
            match mt_server_type_id {
                1 => "tar.gz",
                2 => "zip",
                _ => {
                    return Err(common::error_message(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ssh.unknownServerType",
                    ));
                }
            }
        )
    } else {
        &name_array[0]
    };

    let temp_name = if zip_flag {
        format!("{}_{}", Utc::now().format("%Y%m%d%H%M%S").to_string(), name)
    } else {
        name.to_string()
    };

    if zip_flag {
        let zip_command = match mt_server_type_id {
            1 => {
                let quoted_names = name_array.join("\" \"");
                format!(
                    r#"cd "{}" && tar -czf "{}" "{}""#,
                    dir_path, temp_name, quoted_names
                )
            }
            2 => {
                let paths = name_array
                    .iter()
                    .map(|n| format!(r#"'{}\{}'"#, dir_path, n))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    r#"powershell -Command "Compress-Archive -Path {} -DestinationPath '{}\{}' -Force""#,
                    paths, dir_path, temp_name
                )
            }
            _ => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ssh.unknownServerType",
                ));
            }
        };
        run_ssh_command(&session, &zip_command)?;
    }

    let cat_command = match mt_server_type_id {
        1 => format!(r#"cat "{}/{}""#, dir_path, temp_name),
        2 => format!(
            r#"powershell -Command "Get-Content -Path '{}\{}' -Encoding Byte -ReadCount 0""#,
            dir_path, temp_name
        ),
        _ => {
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ssh.unknownServerType",
            ));
        }
    };

    let stream = run_ssh_stream_command(&session, &cat_command).map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "ssh.streamFailed")
    })?;

    if zip_flag {
        let cleanup_command = match mt_server_type_id {
            1 => format!(r#"rm -f "{}/{}""#, dir_path, temp_name),
            2 => format!(
                r#"powershell -Command "Remove-Item -Path '{}\{}' -Force""#,
                dir_path, temp_name
            ),
            _ => String::new(),
        };
        run_ssh_command(&session, &cleanup_command)?;
    }

    let body = Body::from_bytes_stream(ReaderStream::new(stream));
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            "Content-Disposition",
            format!(r#"attachment; filename="{}""#, name),
        )
        .header("Content-Type", "application/octet-stream")
        .body(body);

    Ok(response)
}

#[handler]
pub async fn add_file(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Json(entry_ext_server_file): Json<EntryExternalServerFile>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    if !is_valid_filename(&entry_ext_server_file.nm) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidFilename",
        ));
    }

    let mut dir_path = entry_ext_server_file.dir.join("/");
    if !is_valid_directory_path(&dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    let entity_exists = check_entity_exists(
        &session,
        mt_server_type_id,
        &format!("{}/{}", dir_path, entry_ext_server_file.nm),
    )?;

    if !entity_exists {
        let command = match mt_server_type_id {
            1 => format!(
                "printf \"%s\" \"{}\" > \"{}/{}\"",
                entry_ext_server_file.content.replace('"', "\\\""),
                dir_path,
                entry_ext_server_file.nm
            ),
            2 => format!(
                "powershell -Command \"$Content = @\"{}\"@; Set-Content -Path '{}' -Value $Content\"",
                entry_ext_server_file
                    .content
                    .replace('`', "``") // Escape backtick
                    .replace('"', "`\"") // Escape double quotes
                    .replace('$', "`$")
                    .replace("@", "`@"),
                format!("{}/{}", dir_path, entry_ext_server_file.nm)
            ),
            _ => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ssh.unknownServerType",
                ));
            }
        };

        let _ = run_ssh_command(&session, &command)?;
        Ok(StatusCode::CREATED)
    } else {
        Err(common::error_message(
            StatusCode::CONFLICT,
            "ssh.fileAlreadyExist",
        ))
    }
}

#[handler]
pub async fn update_file(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Json(entry_ext_server_file): Json<EntryExternalServerFile>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    if !is_valid_filename(&entry_ext_server_file.nm) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidFilename",
        ));
    }

    let mut dir_path = entry_ext_server_file.dir.join("/");
    if !is_valid_directory_path(&dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    let entity_exists = check_entity_exists(
        &session,
        mt_server_type_id,
        &format!("{}/{}", dir_path, entry_ext_server_file.nm),
    )?;

    if entity_exists {
        let command = match mt_server_type_id {
            1 => format!(
                "printf \"%s\" \"{}\" > \"{}/{}\"",
                entry_ext_server_file.content.replace('"', "\\\""),
                dir_path,
                entry_ext_server_file.nm
            ),
            2 => format!(
                "powershell -Command \"$Content = @\"{}\"@; Set-Content -Path '{}' -Value $Content\"",
                entry_ext_server_file
                    .content
                    .replace('`', "``") // Escape backtick
                    .replace('"', "`\"") // Escape double quotes
                    .replace('$', "`$")
                    .replace("@", "`@"),
                format!("{}/{}", dir_path, entry_ext_server_file.nm)
            ),
            _ => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ssh.unknownServerType",
                ));
            }
        };

        run_ssh_command(&session, &command)?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(poem::Error::from_status(StatusCode::NOT_FOUND))
    }
}

#[handler]
pub async fn upload_files(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    let mut dir_array: Vec<String> = Vec::new();
    let mut file_array: Vec<(String, Vec<u8>)> = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();

        if name == "directory" {
            dir_array.push(field.text().await?.to_string());
        } else if name == "files" {
            if let Some(file_name) = field.file_name().map(|s| s.to_string()) {
                let data = field.bytes().await?.to_vec();
                file_array.push((file_name, data));
            }
        }
    }

    let mut dir_path = dir_array.join("/");
    if !is_valid_directory_path(&dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    for (file_name, content) in file_array {
        let mut new_name = file_name.to_string();

        loop {
            let exists = check_entity_exists(
                &session,
                mt_server_type_id,
                &format!("{}/{}", dir_path, new_name),
            )?;

            if exists {
                new_name = generate_copy_name(&new_name);
            } else {
                match mt_server_type_id {
                    1 => {
                        let mut remote_file = session
                            .scp_send(
                                std::path::Path::new(&format!("{}/{}", dir_path, new_name)),
                                0o644,
                                content.len() as u64,
                                None,
                            )
                            .map_err(|_| {
                                common::error_message(
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    "ssh.uploadFailed",
                                )
                            })?;

                        remote_file.write_all(&content).map_err(|_| {
                            common::error_message(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "ssh.writeFailed",
                            )
                        })?;
                    }
                    2 => {
                        let base64_str = general_purpose::STANDARD.encode(content);
                        let command = format!(
                            "$data = \"{}\"; [IO.File]::WriteAllBytes(\"{}\", [Convert]::FromBase64String($data))",
                            base64_str,
                            &format!("{}/{}", dir_path, new_name)
                        );

                        run_ssh_command(&session, &command)?;
                    }
                    _ => {
                        return Err(common::error_message(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "ssh.unknownServerType",
                        ));
                    }
                };
                break;
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[handler]
pub async fn clone_entity(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Json(multiple_ext_server_file): Json<MultipleExternalServerFile>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    let mut dir_path = multiple_ext_server_file.dir.join("/");
    if !is_valid_directory_path(&dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    for file_name in &multiple_ext_server_file.nm {
        let mut new_name = generate_copy_name(file_name);

        loop {
            let exists = check_entity_exists(
                &session,
                mt_server_type_id,
                &format!("{}/{}", dir_path, new_name),
            )?;

            if exists {
                new_name = generate_copy_name(&new_name);
            } else {
                let command = match mt_server_type_id {
                    1 => format!(
                        "cp -r \"{}/{}\" \"{}/{}\"",
                        dir_path, file_name, dir_path, new_name
                    ),
                    2 => format!(
                        "Copy-Item -Path \"{}/{}\" -Destination \"{}/{}\" -Recurse",
                        dir_path, file_name, dir_path, new_name
                    ),
                    _ => {
                        return Err(common::error_message(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "ssh.unknownServerType",
                        ));
                    }
                };

                run_ssh_command(&session, &command)?;
                break;
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[handler]
pub async fn copy_entity(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Json(multiple_ext_server_file): Json<MultipleExternalServerEntity>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    let mut source_dir_path = multiple_ext_server_file.source_dir.join("/");
    if !is_valid_directory_path(&source_dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let mut target_dir_path = multiple_ext_server_file.target_dir.join("/");
    if !is_valid_directory_path(&target_dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 {
        if !source_dir_path.starts_with('/') {
            source_dir_path = format!("/{}", source_dir_path);
        }

        if !target_dir_path.starts_with('/') {
            target_dir_path = format!("/{}", target_dir_path);
        }
    }

    for file_name in &multiple_ext_server_file.nm {
        let mut new_name = file_name.to_string();

        loop {
            let exists = check_entity_exists(
                &session,
                mt_server_type_id,
                &format!("{}/{}", target_dir_path, new_name),
            )?;

            if exists {
                new_name = generate_copy_name(&new_name);
            } else {
                let command = match mt_server_type_id {
                    1 => format!(
                        "cp -r \"{}/{}\" \"{}/{}\"",
                        source_dir_path, file_name, target_dir_path, new_name
                    ),
                    2 => format!(
                        "Copy-Item -Path \"{}/{}\" -Destination \"{}/{}\" -Recurse",
                        source_dir_path, file_name, target_dir_path, new_name
                    ),
                    _ => {
                        return Err(common::error_message(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "ssh.unknownServerType",
                        ));
                    }
                };

                run_ssh_command(&session, &command)?;
                break;
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[handler]
pub async fn remove_entity(
    pool: Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_server_id): Path<i64>,
    Json(multiple_ext_server_file): Json<MultipleExternalServerFile>,
) -> Result<impl IntoResponse> {
    validate_id(ext_server_id)?;

    let mut dir_path = multiple_ext_server_file.dir.join("/");
    if !is_valid_directory_path(&dir_path) {
        return Err(common::error_message(
            StatusCode::BAD_REQUEST,
            "error.invalidDirectory",
        ));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let (session, mt_server_type_id) = create_ssh_session(conn, ext_server_id)?;

    if mt_server_type_id == 1 && !dir_path.starts_with('/') {
        dir_path = format!("/{}", dir_path);
    }

    let file_paths: Vec<String> = multiple_ext_server_file
        .nm
        .iter()
        .map(|f| format!(r#""{}/{}""#, dir_path, f))
        .collect();

    let command = match mt_server_type_id {
            1 => file_paths
                .iter()
                .map(|p| format!(r#"find {} -maxdepth 0 \( -type f -o -type d -empty \) -delete"#, p))
                .collect::<Vec<_>>()
                .join(" && "),
            2 => file_paths
                .iter()
                .map(|p| {
                    format!(
                        r#"powershell -Command "Get-Item {} | Where-Object {{ ($_.PSIsContainer -and @(Get-ChildItem $_.FullName).Count -eq 0) -or (-not $_.PSIsContainer) }} | Remove-Item""#,
                        p
                    )
                })
                .collect::<Vec<_>>()
                .join(" ; "),
            _ => {
                return Err(common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ssh.unknownServerType",
                ));
            }
        };

    run_ssh_command(&session, &command)?;
    Ok(StatusCode::NO_CONTENT)
}
