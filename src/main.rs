mod auth;
mod db;
mod err;
mod schema;
mod handlers;

use crate::auth::BackendRudimentary;
use crate::db::MIGRATIONS;
use crate::err::{Error, Result};
use crate::handlers::{
    create_todo_handler, default_todo_handler, delete_todo_handler, edit_todo_handler,
    hello_handler, home_handler, save_todo_handler, toggle_todo_handler,
};
use auth::page_login_check;
use axum::routing::{delete, post};
use axum::{Router, routing::get};
use axum_login::tower_sessions::cookie::time::Duration;
use axum_login::tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use axum_login::{AuthManagerLayerBuilder, login_required};
use diesel::SqliteConnection;
use diesel_migrations::MigrationHarness;
use dotenvy::dotenv;
use snafu::ResultExt;
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

pub struct AppState {
    conn: SqliteConnection,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().with_whatever_context(|err| format!("Failed to load .env: {}", err))?;
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut conn = db::establish_connection(&db_url)
        .with_whatever_context(|err| format!("Failed to connect to in-memory database {err}"))?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|_| Error::DatabaseMigration {})?;

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));

    let backend = BackendRudimentary {};
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = Router::new()
        // .route("/login_success", get(page_login_success))
        .route("/hello", get(hello_handler))
        // .route_layer(login_required!(BackendRudimentary, login_url = "/login"))
        // .route("/login", get(page_login))
        // .route("/login", post(page_login_check))
        // .route("/unimplemented", get(page_unimplemented))
        .route("/", get(home_handler))
        .route("/toggle/{todo_id}", post(toggle_todo_handler))
        .route("/default/{todo_id}", post(default_todo_handler))
        .route("/edit/{todo_id}", post(edit_todo_handler))
        .route("/save/{todo_id}", post(save_todo_handler))
        .route("/create", post(create_todo_handler))
        .route("/delete/{todo_id}", delete(delete_todo_handler))
        .with_state(Arc::new(Mutex::new(AppState { conn })))
        .route_service("/{*wildcard}", ServeDir::new("./static"))
        .layer(auth_layer)
        .layer(LiveReloadLayer::new());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .with_whatever_context(|err| format!("Failed to bind to port 3000: {}", err))?;
    axum::serve(listener, app)
        .await
        .with_whatever_context(|err| format!("Failed to serve: {}", err))?;

    Ok(())
}
