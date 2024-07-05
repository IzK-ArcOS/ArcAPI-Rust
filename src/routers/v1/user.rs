use axum::extract::{Query, State};
use axum::Json;
use axum::routing::{get, post};
use serde::Deserialize;
use crate::AppState;
use crate::routers::extractors::SessionUser;
use crate::routers::v1::utils::{B64ToStrError, from_b64};
use super::schema::DataResponse;

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/properties", get(get_self_properties))
        .route("/properties/update", post(update_self_properties))
        .route("/rename", get(rename_self))
        .route("/changepswd", get(change_self_password))
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
