use std::io::ErrorKind;
use std::path::PathBuf;
use std::str::FromStr;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use crate::{AppState, db};
use crate::filesystem::{FSError, UserScopedFS};
use crate::routers::extractors::SessionUser;
use super::schema::{NewUser, SelfUser};

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/me", get(get_self_properties).put(set_self_properties).delete(delete_self))
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


async fn delete_self(
    State(AppState { conn_pool, filesystem, .. }): State<AppState>,
    SessionUser(mut user): SessionUser,
) {
    let usfs = UserScopedFS::new(&filesystem, user.id).await.unwrap();  // i hope this doesnt ever fail
    
    tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        user.delete(conn);
    }).await.unwrap();
    
    match usfs.remove_item(&PathBuf::from_str(".").unwrap()).await {
        Err(FSError::HFS(err)) if err.kind() == ErrorKind::NotFound => {},
        whatever => whatever.unwrap()   // i really hope this doesnt fail under most other circumstances as well
    }
}


async fn get_self_properties(
    SessionUser(user): SessionUser
) -> Json<SelfUser> {
    Json(SelfUser {
        id: user.id,
        properties: user.map_properties_as_json()
            .expect("token is valid, so user shouldn't be deleted")
            .expect("user properties should be a valid json"),
        username: user.username.expect("token is valid, so user shouldn't be deleted"),
    })
}


async fn set_self_properties(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(mut user): SessionUser,
    Json(new_prop): Json<serde_json::Value>
) {
    tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();

        user.set_properties(conn, new_prop)
            .expect("token is valid, so user shouldn't be deleted");
    });
}
