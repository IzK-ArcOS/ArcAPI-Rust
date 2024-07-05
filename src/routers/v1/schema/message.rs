use chrono::NaiveDateTime;
use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use crate::db;

#[derive(Deserialize, Serialize)]
pub struct MessagePreview {
    id: i32,
    sender: String,
    receiver: String,
    #[serde(rename = "replyingTo")]
    replying_to: Option<i32>,
    timestamp: u64,
    #[serde(rename = "partialBody")]
    partial_body: String,
    read: bool
}


#[derive(Debug)]
pub enum ConversionError {
    ItemIsDeleted
}


impl MessagePreview {
    pub fn from_msg(conn: &mut SqliteConnection, message: &db::Message, preview_length: usize) -> Result<Self, ConversionError> {
        Ok(Self {
            id: message.id,
            sender: message.get_sender(conn).get_username(),
            receiver: message.get_receiver(conn).get_username(),
            replying_to: message.replying_id,
            timestamp: message.sent_time.signed_duration_since(NaiveDateTime::UNIX_EPOCH).num_milliseconds() as u64,
            partial_body: message.body.clone().ok_or(ConversionError::ItemIsDeleted)?[..preview_length.min(message.body.as_ref().map_or(0, |s| s.len()))].to_string(),
            read: message.is_read.ok_or(ConversionError::ItemIsDeleted)?,
        })
    }
}


#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize)]
pub struct SentMessage {
    id: i32,
    sender: String,
    receiver: String,
    replier: Option<i32>
}


impl SentMessage {
    pub fn from_msg(conn: &mut SqliteConnection, message: &db::Message) -> Self {
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
    id: i32,
    sender: String,
    receiver: String,
    body: String,
    replies: Vec<i32>,
    #[serde(rename = "replyingTo")]
    replying_to: Option<i32>,
    timestamp: u64,
    read: bool
}


impl Message {
    pub fn from_msg(conn: &mut SqliteConnection, message: &db::Message) -> Result<Self, ConversionError> {
        Ok(Self {
            id: message.id,
            sender: message.get_sender(conn).get_username(),
            receiver: message.get_receiver(conn).get_username(),
            body: message.body.clone().ok_or(ConversionError::ItemIsDeleted)?,
            replies: db::Message::get_all_not_deleted_replies_for_msg(conn, message).into_iter().map(|msg| msg.id).collect(),
            replying_to: message.replying_id,
            timestamp: message.sent_time.signed_duration_since(NaiveDateTime::UNIX_EPOCH).num_milliseconds() as u64,
            read: message.is_read.ok_or(ConversionError::ItemIsDeleted)?,
        })
    }
}
