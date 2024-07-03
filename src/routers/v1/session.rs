use std::time::Duration;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::routing::get;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Basic;
use axum_extra::TypedHeader;
use crate::{AppState, db};
use crate::routers::extractors::SessionToken;
use super::schema::{DataResponse, Session};

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/auth", get(create_session))
        .route("/logoff", get(delete_session))
}


async fn create_session(
    State(AppState { conn_pool, config }): State<AppState>,
    TypedHeader(basic_creds): TypedHeader<Authorization<Basic>>
) -> Result<Json<DataResponse<Session>>, StatusCode> {
    let session = tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        let token = db::Token::auth(conn, 
                                    basic_creds.username(), 
                                    basic_creds.password(), 
                                    config.auth.session_lifetime.map(Duration::from_secs));
        
        token.map(|t| Session::from_token(conn, t))
    }).await.unwrap();

    match session {
        None => Err(StatusCode::UNAUTHORIZED),
        Some(s) => Ok(Json(DataResponse::new(s)))
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
