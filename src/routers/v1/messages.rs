use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
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
use crate::routers::v1::schema::{DataResponse, Message, MessagePreview, MessageThreadPart, SentMessage};

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/list", get(list_messages))
        .route("/send", post(send_message))
        .route("/reply", post(send_reply))
        .route("/get", get(get_message))
        .route("/delete", get(delete_message))
        .route("/thread", get(get_thread))
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
            |msg| MessagePreview::new(conn, &msg, preview_length.unwrap_or(80) as usize)
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

        Ok(SentMessage::new(conn, &msg))
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

        Ok(SentMessage::new(conn, &msg))
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
            Self::MessageNotAccessibleError => (StatusCode::FORBIDDEN, "you do not have such access to this message").into_response(),
            Self::InvalidID(parse_err) => (StatusCode::BAD_REQUEST, format!("the ID is invalid: {parse_err}")).into_response(),
            Self::MessageIsDeleted => (StatusCode::GONE, "the message is deleted").into_response(),
        }
    }
}


async fn get_message(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(MsgGet { id: msg_id_enc }): Query<MsgGet>,
) -> Result<Json<DataResponse<Message>>, GetMessageError> {
    let msg_id = from_b64(&msg_id_enc).map_err(GetMessageError::B64DecodeError)?.parse().map_err(GetMessageError::InvalidID)?;

    // xxx would it be better to split up one large such blocking task, or leave as is? (more like how would it better for async pattern)
    let message = tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();

        let mut msg = db::Message::get(conn, msg_id).ok_or(GetMessageError::MessageNotFoundError)?;

        if !msg.is_accessible_to(&user) {
            return Err(GetMessageError::MessageNotAccessibleError);
        };
        
        if user.id == msg.receiver_id {
            let _ = msg.mark_as_read(conn);
        }

        Ok(Message::new(conn, &msg))
    }).await.unwrap()?;
    
    Ok(Json(DataResponse::new(message)))
}


async fn delete_message(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(MsgGet { id: msg_id_enc }): Query<MsgGet>,
) -> Result<(), GetMessageError> {
    let msg_id = from_b64(&msg_id_enc).map_err(GetMessageError::B64DecodeError)?.parse().map_err(GetMessageError::InvalidID)?;

    tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();

        let mut msg = db::Message::get(conn, msg_id).ok_or(GetMessageError::MessageNotFoundError)?;

        if !user.id == msg.sender_id {
            return Err(GetMessageError::MessageNotAccessibleError);
        };

        msg.delete(conn);

        Ok(())
    }).await.unwrap()?;

    Ok(())
}


async fn get_thread(
    State(AppState { conn_pool, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(MsgGet { id: msg_id_enc }): Query<MsgGet>,
) -> Result<Json<DataResponse<MessageThreadPart>>, GetMessageError> {
    let msg_id = from_b64(&msg_id_enc).map_err(GetMessageError::B64DecodeError)?.parse().map_err(GetMessageError::InvalidID)?;

    // i am sorry for this code, but hey it should be fast! ...i hope
    let msg_thread = tokio::task::spawn_blocking(move || {
        let conn = &mut conn_pool.get().unwrap();

        let msg = db::Message::get(conn, msg_id).ok_or(GetMessageError::MessageNotFoundError)?;

        if !msg.is_accessible_to(&user) {
            return Err(GetMessageError::MessageNotAccessibleError);
        };

        let (thread_parts_ids, root_msg_id, messages) = {
            let mut thread_parts: HashMap<i32, Vec<i32>> = HashMap::new();  // an {id: [msg_which_reply_to_id]} map
            let mut messages: HashMap<i32, db::Message> = HashMap::new();  // used for caching
            let mut ids_to_resolve_queue = vec![msg.id];

            messages.insert(msg.id, msg);
            let mut resolved_ids = HashSet::new();
            let mut root_msg_id = None;

            while let Some(msg_id) = ids_to_resolve_queue.pop() {
                // xxx is this check even needed?
                if resolved_ids.contains(&msg_id) {
                    continue;
                };

                let msg = messages.entry(msg_id).or_insert_with(|| db::Message::get(conn, msg_id).expect("the queue stores only existing IDs"));

                if let Some(replying_msg) = msg.get_replying_msg(conn) {
                    if !resolved_ids.contains(&replying_msg.id) {
                        let thread_part = thread_parts.entry(replying_msg.id).or_default();

                        thread_part.push(msg_id);

                        ids_to_resolve_queue.push(replying_msg.id);

                        // todo somehow cache this
                        // messages.insert(replying_msg.id, replying_msg);
                    };
                } else {
                    if root_msg_id.is_none() {
                        root_msg_id = Some(msg_id);
                    } else {
                        unreachable!("there cannot be multiple message roots");
                    };
                };

                for reply_msg in msg.get_all_not_deleted_replies(conn) {
                    if !reply_msg.is_accessible_to(&user) || resolved_ids.contains(&reply_msg.id) {
                        continue;
                    };

                    let thread_part = thread_parts.entry(msg_id).or_default();

                    thread_part.push(reply_msg.id);

                    ids_to_resolve_queue.push(reply_msg.id);

                    messages.insert(reply_msg.id, reply_msg);
                };

                resolved_ids.insert(msg_id);
            };

            (thread_parts, root_msg_id.unwrap(), messages)
        };

        let mut thread_parts = {
            let mut thread_parts: HashMap<i32, Arc<Mutex<MessageThreadPart>>> = HashMap::new();
            let mut get_thread_part = |id: i32| {
                thread_parts.entry(id).or_insert_with(|| Arc::new(Mutex::new(
                    MessageThreadPart::new_partial(
                        conn,
                        messages.get(&id)
                            .expect("all messages should have gotten cached"),
                        80
                    )
                ))).clone()
            };

            for (msg_id, replies_ids) in thread_parts_ids.iter() {
                let thread_part = get_thread_part(*msg_id);

                thread_part.lock().unwrap().replies.append(&mut replies_ids.iter().map(|id| get_thread_part(*id)).collect());
            };

            thread_parts
        };

        Ok(Arc::<_>::try_unwrap(thread_parts.remove(&root_msg_id).unwrap())
            .expect("it's the root message, so it should not have any references to it")
            .into_inner().unwrap())
    }).await.unwrap()?;

    Ok(Json(DataResponse::new(msg_thread)))
}
