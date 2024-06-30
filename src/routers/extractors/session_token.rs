use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::IntoResponse
};
use axum::http::StatusCode;
use axum::response::Response;
use axum_extra::{
    headers::{
        Authorization,
        authorization::Bearer
    },
    typed_header::TypedHeaderRejection,
    TypedHeader
};
use crate::{AppState, db};

pub struct SessionToken(pub db::Token);


#[derive(Debug)]
pub enum SessionTokenRejection {
    HeaderRejection(TypedHeaderRejection),
    InvalidToken
}


impl std::fmt::Display for SessionTokenRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HeaderRejection(rej) => write!(f, "Header is invalid: {rej}"),
            Self::InvalidToken => write!(f, "Token is invalid"),
        }
    }
}


impl IntoResponse for SessionTokenRejection {
    fn into_response(self) -> Response {
        match self {
            rej @ Self::HeaderRejection(_) => (StatusCode::BAD_REQUEST, rej.to_string()),
            it @ Self::InvalidToken => (StatusCode::UNAUTHORIZED, it.to_string()),
        }.into_response()
    }
}


#[axum::async_trait]
impl FromRequestParts<AppState> for SessionToken {
    type Rejection = SessionTokenRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState { conn_pool }: &AppState
    ) -> Result<Self, Self::Rejection> {
        let token_value =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &()).await
                .map_err(SessionTokenRejection::HeaderRejection)?;

        let conn_pool = conn_pool.clone();
        let token = tokio::task::spawn_blocking(move || {
            let mut conn = conn_pool.get().unwrap();
            
            match db::Token::get(&mut conn, token_value.token()) {
                Some(t) if !t.is_valid() => { t.delete(&mut conn); None }
                token => token
            }
        }).await.unwrap();

        Ok(Self(token.ok_or(SessionTokenRejection::InvalidToken)?))
    }
}
