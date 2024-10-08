use axum::extract::State;
use axum::routing::get;
use crate::{AppState, db};
use super::schema::{DataResponse, PartialUser};

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/get", get(get_all_users))
}


async fn get_all_users(
    State(AppState { conn_pool, .. }): State<AppState>
) -> axum::Json<DataResponse<Vec<PartialUser>>> {
    let users = tokio::task::spawn_blocking(move || {
        let mut conn = conn_pool.get().unwrap();
        
        db::User::get_all_accessible(&mut conn) 
    }).await.unwrap();
    
    let partial_users = users.into_iter()
        .filter_map(|u| PartialUser::try_new(&u).ok())
        .collect::<Vec<_>>();
    
    axum::Json(DataResponse::new(partial_users))
}
