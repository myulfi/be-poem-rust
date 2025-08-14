use rand::Rng;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use validator::ValidationErrors;

use poem::{IntoResponse, Response, Result, error::Error, http::StatusCode, web::Json};
use serde::Serialize;

use crate::utils::common;

#[derive(Serialize)]
pub struct MessageResponse {
    message: String,
}

pub fn error_message(status: StatusCode, msg: &str) -> Error {
    Error::from_response(
        (
            status,
            Json(MessageResponse {
                message: msg.to_string(),
            }),
        )
            .into_response(),
    )
}

// pub fn error_message(status: StatusCode, message: impl Into<String>) -> Response {
//     let msg = message.into();
//     (status, msg).into_response()
// }

pub fn generate_id() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_millis(0))
        .as_millis();

    let random = rand::thread_rng().gen_range(0..=999) as u128;
    let id = now * 1000 + random;
    let max_safe: u128 = i64::MAX as u128;
    let safe_id = if id > max_safe { max_safe } else { id };
    safe_id as i64
}

pub fn validate_id(id: i64) -> Result<()> {
    if id >= 1_000_000_000_000_000 && id <= 9_999_999_999_999_999 {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND.into())
    }
}

pub fn validate_ids(ids: &str) -> Result<()> {
    if ids.len() % 16 == 0 && ids.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND.into())
    }
}

pub fn parse_ids_from_string(input: &str) -> Result<Vec<i64>> {
    let mut ids = Vec::new();

    for chunk in input.as_bytes().chunks(16) {
        let id_str = std::str::from_utf8(chunk)
            .map_err(|_| common::error_message(StatusCode::BAD_REQUEST, "Invalid UTF-8 ID"))?;

        let id = id_str
            .parse::<i64>()
            .map_err(|_| common::error_message(StatusCode::BAD_REQUEST, "Invalid ID number"))?;

        ids.push(id);
    }

    Ok(ids)
}

pub fn validation_error_response(e: ValidationErrors) -> poem::Error {
    let mut details = serde_json::Map::new();

    for (field, errors) in e.field_errors().iter() {
        let messages: Vec<String> = errors
            .iter()
            .filter_map(|err| err.message.as_ref())
            .map(|msg| msg.to_string())
            .collect();

        details.insert(field.to_string(), json!(messages));
    }

    let body = json!({
        "error": "Validation failed",
        "details": details
    });

    poem::Error::from_response(
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .content_type("application/json")
            .body(body.to_string()),
    )
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
