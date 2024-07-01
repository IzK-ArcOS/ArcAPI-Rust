use axum::extract::State;
use axum::Json;
use axum::routing::{get, post};
use crate::AppState;
use crate::routers::extractors::SessionUser;
use super::schema::DataResponse;

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/properties", get(get_self_properties))
        .route("/properties/update", post(update_self_properties))
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
    State(AppState { conn_pool }): State<AppState>,
    SessionUser(mut user): SessionUser,
    Json(new_prop): Json<serde_json::Value>
) {
    tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        user.set_properties(conn, new_prop)
            .expect("token is valid, so user shouldn't be deleted");
    });
}
