use std::time::Duration;
use axum::extract::{State};
use axum::{Json};
use axum::http::StatusCode;
use axum::routing::post;
use axum_typed_multipart::TypedMultipart;
use crate::{AppState, db};
use crate::routers::extractors::SessionToken;
use super::schema::{NewSession, Session};

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(create_session).delete(delete_session))
}


async fn create_session(
    State(AppState { conn_pool, config, .. }): State<AppState>,
    TypedMultipart(NewSession { username, password }): TypedMultipart<NewSession>  // todo somehow make it support both multipart and form data
) -> Result<Json<Session>, StatusCode> {
    let token = tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        db::Token::auth(conn, &username, &password, config.auth.session_lifetime.map(Duration::from_secs))
    }).await.unwrap();
    
    match token {
        None => Err(StatusCode::UNAUTHORIZED),
        Some(t) => Ok(Json(Session::from(t)))
    }
}


async fn delete_session(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionToken(token): SessionToken
) {
    tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        token.delete(conn);
    }).await.unwrap();
}
