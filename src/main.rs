mod auth;
mod db;
mod err;
mod handlers;

use crate::auth::BackendRudimentary;
use crate::err::Result;
use crate::handlers::{
    handler_create_todo, handler_delete_todo, handler_get_one_todo, handler_home, handler_login,
    handler_login_success, handler_save_todo, handler_todo_edit, handler_toggle_todo,
    hello_handler,
};
use auth::handler_login_check;
use axum::routing::{delete, post};
use axum::{Router, routing::get};
use axum_login::tower_sessions::cookie::time::Duration;
use axum_login::tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use axum_login::{AuthManagerLayerBuilder, login_required};
use dotenvy::dotenv;
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

pub struct AppState {
    conn: rusqlite::Connection,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut conn = db::establish_connection(&db_url)?;
    db::run_migrations(&mut conn)?;

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));

    let backend = BackendRudimentary {};
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = Router::new()
        .route("/login_success", get(handler_login_success))
        // Every route above this needs authentication
        .route_layer(login_required!(BackendRudimentary, login_url = "/login"))
        // Every route below this doesn't need authentication
        .route("/hello", get(hello_handler))
        .route("/login", get(handler_login))
        .route("/login", post(handler_login_check))
        // .route("/unimplemented", get(page_unimplemented))
        .route("/", get(handler_home))
        .route("/toggle/{todo_id}", post(handler_toggle_todo))
        .route("/default/{todo_id}", post(handler_get_one_todo))
        .route("/edit/{todo_id}", post(handler_todo_edit))
        .route("/save/{todo_id}", post(handler_save_todo))
        .route("/create", post(handler_create_todo))
        .route("/delete/{todo_id}", delete(handler_delete_todo))
        .with_state(Arc::new(Mutex::new(AppState { conn })))
        .route_service("/{*wildcard}", ServeDir::new("./static"))
        .layer(auth_layer)
        .layer(LiveReloadLayer::new());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
