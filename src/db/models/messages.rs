use chrono::NaiveDateTime;
use diesel::prelude::*;
use super::super::schema::{self, messages::dsl::*};


#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Message {
    pub id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub body: String,
    pub replying_id: i32,
    pub sent_time: NaiveDateTime,
    pub is_read: bool,
    pub is_deleted: bool,
}
