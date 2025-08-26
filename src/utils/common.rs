use rand::Rng;
use serde_json::json as json_macro;
use std::time::{SystemTime, UNIX_EPOCH};
use validator::ValidationErrors;

use crate::{models::common::Pagination, utils::common};
use poem::{IntoResponse, Response, Result, error::Error, http::StatusCode, web::Json};
use serde::Serialize;

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

        details.insert(field.to_string(), json_macro!(messages));
    }

    let body = json_macro!({
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

pub fn parse_pagination(pagination: &Pagination) -> (i64, i64) {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);
    (start, length)
}

pub fn encode_special_chars(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if !c.is_alphanumeric() && c.is_ascii() {
                format!("%{:02X}", c as u8)
            } else {
                c.to_string()
            }
        })
        .collect()
}
