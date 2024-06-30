use axum::routing::get;
use crate::AppState;
use crate::routers::extractors::SessionUser;
use super::schema::DataResponse;

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/properties", get(get_self_properties))
}



async fn get_self_properties(
    SessionUser(user): SessionUser
) -> axum::Json<DataResponse<serde_json::Value>> {
    axum::Json(DataResponse::new(
        user.map_properties_as_json()
            .expect("token is valid, so user shouldn't be deleted")
            .expect("user properties should be a valid json")
    ))
}
