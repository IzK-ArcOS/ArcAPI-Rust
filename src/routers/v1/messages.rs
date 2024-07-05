use axum::extract::{Query, State};
use axum::Json;
use axum::routing::get;
use serde::Deserialize;
use crate::{AppState, db};
use crate::routers::extractors::SessionUser;
use crate::routers::v1::schema::{DataResponse, MessagePreview};

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/list", get(list_messages))
}


#[derive(Deserialize)]
struct LLPagination {
    count: Option<i64>,
    offset: Option<u64>,
    descending: Option<bool>,
}


#[derive(Deserialize)]
struct MsgPreviewCfg {
    preview_length: Option<u32>,
}


async fn list_messages(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(LLPagination { count, offset, descending }): Query<LLPagination>,
    Query(MsgPreviewCfg { preview_length }): Query<MsgPreviewCfg>,
) -> Json<DataResponse<Vec<MessagePreview>>> {
    let message_previews = tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();

        // todo maybe combine those 2 operations into a single, so that we would avoid doing so many db operations in a loop
        let messages = db::Message::get_all_not_deleted_accessible_to_user(conn, &user, 
                                                            descending.unwrap_or(true), 
                                                            count.unwrap_or(-1), 
                                                            offset.unwrap_or(0));
        
        messages.into_iter().map(
            |msg| MessagePreview::from_msg(conn, &msg, preview_length.unwrap_or(80) as usize)
                .expect("messages are not deleted, as we have filtered out the deleted once before")
        ).collect::<Vec<_>>()
    }).await.unwrap();
    
    Json(DataResponse::new(message_previews))
}
