use rand::Rng;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use validator::ValidationErrors;

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
        .unwrap()
        .as_millis();

    let random = rand::thread_rng().gen_range(1..=999);

    // Combine timestamp and random number
    format!("{}{:03}", now, random).parse().unwrap()
}

pub fn validate_id(id: i64) -> Result<()> {
    if id >= 1_000_000_000_000_000 && id <= 9_999_999_999_999_999 {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND.into())
    }
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
