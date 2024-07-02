use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use crate::{AppState, db};
use super::schema::NewUser;

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(create_new_user))
}


enum UserCreationError {
    DbError(db::UserCreationError),
}


impl IntoResponse for UserCreationError {
    fn into_response(self) -> Response {
        match self {
            Self::DbError(db::UserCreationError::SuchUsernameIsAlreadyUsed) => StatusCode::CONFLICT.into_response()
        }
    }
}



async fn create_new_user(
    State(AppState { conn_pool, .. }): State<AppState>,
    Json(NewUser { username, password, properties }): Json<NewUser>
) -> Result<String, UserCreationError> {
    Ok(tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        db::User::create(conn, &username, &password, properties.as_ref())
    }).await.unwrap().map_err(UserCreationError::DbError)?.id.to_string())
}
