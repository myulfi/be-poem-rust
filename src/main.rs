mod auth;
mod db;
mod models;
mod routes;
mod schema;
mod utils;

use anyhow::Result;
use dotenvy::dotenv;
use poem::http::StatusCode;
use poem::web::{Json, Path};
use poem::{EndpointExt, Route, Server, get, handler, listener::TcpListener};
use poem::{IntoResponse, Response, post};
use serde_json::Value;
use std::env;
use std::net::SocketAddr;
use tokio::{fs, signal};

use crate::auth::jwt::{generate_token, refresh_token};
use crate::routes::test::test_routes;

// use std::io::{self, Write};
// io::stdout().flush().unwrap();

#[handler]
// fn hello() -> &'static str {
//     "Hello, world!"
// }
fn hello() -> poem::Response {
    let app_name = env::var("APP_NAME").unwrap_or_else(|_| "World".to_string());
    format!("Hello, {}!", app_name).into_response()
}

#[handler]
async fn get_language(Path(lng): Path<String>) -> Response {
    let path = format!("src/language/{}.json", lng);

    match fs::read_to_string(&path).await {
        Ok(data) => match serde_json::from_str::<Value>(&data) {
            Ok(json) => Json(json).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Invalid JSON").into_response(),
        },
        Err(_) => (StatusCode::NOT_FOUND, "Could not read language file").into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL is not set in environment"))?;
    let pool = db::init_pool(&database_url)?;

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .map_err(|_| anyhow::anyhow!("PORT must be a valid number"))?;

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let app = Route::new()
        .at("", get(hello))
        .at("/:lng/language.json", get(get_language))
        .nest("/generate-token.json", post(generate_token))
        .nest("/refresh-token.json", post(refresh_token))
        .nest("/test", test_routes())
        .data(pool);

    let listener = TcpListener::bind(addr);

    println!("Server running at http://localhost:{}", port);
    Server::new(listener)
        .run_with_graceful_shutdown(
            app,
            async {
                signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
                println!("\n🛑 Received Ctrl+C, shutting down gracefully...");
            },
            None,
        )
        .await?;

    println!("✅ Server stopped.");
    Ok(())
}
