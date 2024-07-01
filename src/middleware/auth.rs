use axum::extract::{Query, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use serde::Deserialize;
use crate::AppState;


const UNPROTECTED_ROUTES: &[&str] = &["/connect"];


#[derive(Debug, Deserialize)]
pub struct AuthCode {
    pub ac: String,
}


pub async fn verify_auth_code(
    State(AppState { config, .. }): State<AppState>,
    ac: Option<Query<AuthCode>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !UNPROTECTED_ROUTES.contains(&request.uri().path()) {
        if let Some(ref valid_ac) = config.auth.code {
            match ac {
                None =>
                    return Err(StatusCode::UNAUTHORIZED),
                Some(Query(AuthCode { ac })) if *valid_ac != ac =>
                    return Err(StatusCode::UNAUTHORIZED),
                _ => {}
            }
        };
    };
    
    Ok(next.run(request).await)
}
