use std::num::ParseIntError;
use axum::extract::{Query, State, };
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use crate::{AppState, db};
use crate::routers::extractors::SessionUser;
use crate::routers::v1::utils::{B64ToStrError, from_b64};
use crate::routers::v1::schema::{DataResponse, Message, MessagePreview, SentMessage};

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/list", get(list_messages))
        .route("/send", post(send_message))
        .route("/reply", post(send_reply))
        .route("/get", get(get_message))
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


enum SendMessageError {
    B64Decoding(B64ToStrError),
    TargetUserNotFound,
    ReplyMessageNotFound,
}


impl IntoResponse for SendMessageError {
    fn into_response(self) -> Response {
        match self {
            Self::B64Decoding(dec_err) => dec_err.into_response(),
            Self::TargetUserNotFound => (StatusCode::NOT_FOUND, "the target user was not found").into_response(),
            Self::ReplyMessageNotFound => (StatusCode::NOT_FOUND, "the reply message was not found").into_response()
        }
    }
}


#[derive(Deserialize)]
struct MsgSend {
    target: String,
}


async fn send_message(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(MsgSend { target: target_username_enc }): Query<MsgSend>,
    contents: String,
) -> Result<Json<DataResponse<SentMessage>>, SendMessageError> {
    let msg = tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        let target_username = from_b64(&target_username_enc).map_err(SendMessageError::B64Decoding)?;
        let target = db::User::get_by_username(conn, &target_username).ok_or(SendMessageError::TargetUserNotFound)?;

        let msg = db::Message::send(conn, &user, &target, None, &contents);

        Ok(SentMessage::from_msg(conn, &msg))
    }).await.unwrap()?;
    
    Ok(Json(DataResponse::new(msg)))
}


#[derive(Deserialize)]
struct MsgReply {
    target: String,
    id: i32
}


// this is just insane amounts of duplicate code
async fn send_reply(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(MsgReply { target: target_username_enc, id: reply_msg_id }): Query<MsgReply>,
    contents: String,
) -> Result<Json<DataResponse<SentMessage>>, SendMessageError> {
    let msg = tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();

        let target_username = from_b64(&target_username_enc).map_err(SendMessageError::B64Decoding)?;
        let target = db::User::get_by_username(conn, &target_username).ok_or(SendMessageError::TargetUserNotFound)?;

        let reply = db::Message::get(conn, reply_msg_id).ok_or(SendMessageError::ReplyMessageNotFound)?;
        
        let msg = db::Message::send(conn, &user, &target, Some(&reply), &contents);

        Ok(SentMessage::from_msg(conn, &msg))
    }).await.unwrap()?;

    Ok(Json(DataResponse::new(msg)))
}


#[derive(Deserialize)]
struct MsgGet {
    id: String,
}


enum GetMessageError {
    B64DecodeError(B64ToStrError),
    InvalidID(ParseIntError),
    MessageNotFoundError,
    MessageNotAccessibleError,
    MessageIsDeleted,
}


impl IntoResponse for GetMessageError {
    fn into_response(self) -> Response {
        match self {
            Self::B64DecodeError(dec_err) => dec_err.into_response(),
            Self::MessageNotFoundError => (StatusCode::NOT_FOUND, "the message was not found").into_response(),
            Self::MessageNotAccessibleError => (StatusCode::FORBIDDEN, "you do not have access to this message").into_response(),
            Self::InvalidID(parse_err) => (StatusCode::BAD_REQUEST, format!("the ID is invalid: {parse_err}")).into_response(),
            Self::MessageIsDeleted => (StatusCode::GONE, "the message is deleted").into_response(),
        }
    }
}


async fn get_message(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(MsgGet { id: msg_id_enc }): Query<MsgGet> 
) -> Result<Json<DataResponse<Message>>, GetMessageError> {
    let message = tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();
        
        let msg_id = from_b64(&msg_id_enc).map_err(GetMessageError::B64DecodeError)?.parse().map_err(GetMessageError::InvalidID)?;
        let mut msg = db::Message::get(conn, msg_id).ok_or(GetMessageError::MessageNotFoundError)?;

        if !msg.is_accessible_to(&user) {
            return Err(GetMessageError::MessageNotAccessibleError);
        };
        
        if user.id == msg.receiver_id {
            msg.mark_as_read(conn).map_err(|_| GetMessageError::MessageIsDeleted)?
        }

        Ok(Message::from_msg(conn, &msg)
            .expect("the message is not deleted, it was already checked"))
    }).await.unwrap()?;
    
    Ok(Json(DataResponse::new(message)))
}
