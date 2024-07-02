use axum::extract::State;
use axum::Json;
use axum::routing::get;
use crate::AppState;
use super::schema::MetaInfo;

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(get_meta_info))
}


async fn get_meta_info(
    State(AppState { config, .. }): State<AppState>
) -> Json<MetaInfo> {
    Json(MetaInfo::from(config.as_ref()))
}
