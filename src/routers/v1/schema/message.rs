use std::sync::{Arc, Mutex};
use chrono::NaiveDateTime;
use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use crate::db;

#[derive(Deserialize, Serialize)]
pub struct MessagePreview {
    pub id: i32,
    pub sender: String,
    pub receiver: String,
    #[serde(rename = "replyingTo")]
    pub replying_to: Option<i32>,
    pub timestamp: u64,
    #[serde(rename = "partialBody")]
    pub partial_body: String,
    pub read: bool
}


#[derive(Debug)]
pub enum ConversionError {
    ItemIsDeleted
}


impl MessagePreview {
    pub fn new(conn: &mut SqliteConnection, message: &db::Message, preview_length: usize) -> Result<Self, ConversionError> {
        Ok(Self {
            id: message.id,
            sender: message.get_sender(conn).get_username(),
            receiver: message.get_receiver(conn).get_username(),
            replying_to: message.replying_id,
            timestamp: message.sent_time.signed_duration_since(NaiveDateTime::UNIX_EPOCH).num_milliseconds() as u64,
            partial_body: message.get_body_preview(preview_length).map_err(|_| ConversionError::ItemIsDeleted)?.to_string(),
            read: message.is_read.ok_or(ConversionError::ItemIsDeleted)?,
        })
    }
}


#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize)]
pub struct SentMessage {
    pub id: i32,
    pub sender: String,
    pub receiver: String,
    pub replier: Option<i32>
}


impl SentMessage {
    pub fn new(conn: &mut SqliteConnection, message: &db::Message) -> Self {
        Self {
            id: message.id,
            sender: message.get_sender(conn).get_username(),
            receiver: message.get_receiver(conn).get_username(),
            replier: message.replying_id
        }
    }
}


#[derive(Deserialize, Serialize)]
pub struct Message {
    pub id: i32,
    pub sender: String,
    pub receiver: String,
    pub body: String,
    pub replies: Vec<i32>,
    #[serde(rename = "replyingTo")]
    pub replying_to: Option<i32>,
    pub timestamp: u64,
    pub read: bool
}


impl Message {
    pub fn new(conn: &mut SqliteConnection, message: &db::Message) -> Self {
        Self {
            id: message.id,
            sender: message.get_sender(conn).get_username(),
            receiver: message.get_receiver(conn).get_username(),
            body: message.body.clone().unwrap_or("[deleted]".to_string()),
            replies: message.get_all_not_deleted_replies(conn).into_iter().map(|msg| msg.id).collect(),
            replying_to: message.replying_id,
            timestamp: message.sent_time.signed_duration_since(NaiveDateTime::UNIX_EPOCH).num_milliseconds() as u64,
            read: message.is_read.unwrap_or(false),
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
pub struct MessageThreadPart {
    pub id: i32,
    pub sender: String,
    pub receiver: String,
    #[serde(rename = "partialBody")]
    pub partial_body: String,
    pub replies: Vec<Arc<Mutex<MessageThreadPart>>>,
    #[serde(rename = "replyingTo")]
    pub replying_to: Option<i32>,
    pub timestamp: u64,
}


impl MessageThreadPart {
    pub fn new_partial(conn: &mut SqliteConnection, message: &db::Message, preview_length: usize) -> Self {
        Self {
            id: message.id,
            sender: message.get_sender(conn).get_username(),
            receiver: message.get_receiver(conn).get_username(),
            partial_body: message.get_body_preview(preview_length).unwrap_or("[deleted]").to_string(),
            replies: Vec::new(),
            replying_to: message.replying_id,
            timestamp: message.sent_time.signed_duration_since(NaiveDateTime::UNIX_EPOCH).num_milliseconds() as u64,
        }
    }
}
