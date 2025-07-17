use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

use poem::{IntoResponse, error::Error, http::StatusCode, web::Json};
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
