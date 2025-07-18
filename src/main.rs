mod auth;
mod db;
mod models;
mod routes;
mod schema;
mod utils;

use anyhow::Result;
use dotenvy::dotenv;
use poem::{EndpointExt, Route, Server, get, handler, listener::TcpListener};
use std::env;
use std::net::SocketAddr;
use tokio::signal;

use crate::auth::jwt::credential_routes;
use crate::routes::test::test_routes;

// use std::io::{self, Write};
// io::stdout().flush().unwrap();

#[handler]
fn hello() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::init_pool(&database_url);

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let app = Route::new()
        .at("/", get(hello))
        .nest("", credential_routes())
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
