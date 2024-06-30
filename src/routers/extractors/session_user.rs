use std::fmt::Formatter;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::{AppState, db};
use crate::routers::extractors::session_token::SessionTokenRejection;
use crate::routers::extractors::SessionToken;

pub struct SessionUser(pub db::User);


#[derive(Debug)]
pub enum SessionUserRejection {
    TokenRejection(SessionTokenRejection),
}


impl std::fmt::Display for SessionUserRejection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenRejection(rej) => <_ as std::fmt::Display>::fmt(rej, f),
        }
    }
}


impl IntoResponse for SessionUserRejection {
    fn into_response(self) -> Response {
        match self {
            Self::TokenRejection(rej) => rej.into_response(),
        }
    }
}


#[axum::async_trait]
impl FromRequestParts<AppState> for SessionUser {
    type Rejection = SessionUserRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState
    ) -> Result<Self, Self::Rejection> {
        // xxx should we "inline" the session token, so that we wouldn't have to get the db conn twice?
        let SessionToken(token) = SessionToken::from_request_parts(parts, app_state).await
            .map_err(SessionUserRejection::TokenRejection)?; 
        
        let conn_pool = app_state.conn_pool.clone();
        let user = tokio::task::spawn_blocking(move || {
            let mut conn = conn_pool.get().unwrap();
            
           token.get_owner(&mut conn) 
        }).await.unwrap();
        
        Ok(Self(user))
    }
}
