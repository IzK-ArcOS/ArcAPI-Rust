use std::io::ErrorKind;
use std::path::PathBuf;
use std::str::FromStr;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Basic;
use axum_extra::TypedHeader;
use serde::Deserialize;
use crate::{AppState, db};
use crate::filesystem::{FSError, UserScopedFS};
use crate::routers::extractors::SessionUser;
use crate::routers::v1::utils::{B64ToStrError, from_b64};
use super::schema::DataResponse;

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/properties", get(get_self_properties))
        .route("/properties/update", post(update_self_properties))
        .route("/rename", get(rename_self))
        .route("/changepswd", get(change_self_password))
        .route("/delete", get(delete_self))
        .route("/create", get(create_new_user))
}



async fn get_self_properties(
    SessionUser(user): SessionUser
) -> Json<DataResponse<serde_json::Value>> {
    Json(DataResponse::new(
        user.map_properties_as_json()
            .expect("token is valid, so user shouldn't be deleted")
            .expect("user properties should be a valid json")
    ))
}


async fn update_self_properties(
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


#[derive(Deserialize)]
struct NewName {
    newname: String
}


async fn rename_self(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(mut user): SessionUser,
    Query(NewName { newname: new_enc }): Query<NewName>,
) -> Result<(), B64ToStrError> {
    let new = from_b64(&new_enc)?;
    
    tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        user.rename(conn, new)
            .expect("token is valid, so user should be as well too");
    }).await.unwrap();
    
    Ok(())
}


#[derive(Deserialize)]
struct NewPassword {
    new: String,
}


async fn change_self_password(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(mut user): SessionUser,
    Query(NewPassword { new: new_enc }): Query<NewPassword>,
) -> Result<(), B64ToStrError> {
    let new = from_b64(&new_enc)?;
    
    tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        user.set_password(conn, &new)
            .expect("token is valid, so user should be as well too");
    }).await.unwrap();
    
    Ok(())
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
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
) -> Result<Json<DataResponse<()>>, UserCreationError> {
    tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        db::User::create(conn, auth.username(), auth.password(), None)
    }).await.unwrap().map_err(UserCreationError::DbError)?;
    
    Ok(Json(DataResponse::new(())))
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
