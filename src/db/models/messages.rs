use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use crate::db;
use super::gen_id;
use super::super::schema::{
    self,
    messages::dsl::{*, is_deleted},
    users::{self, dsl::*}
};


#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Message {
    pub id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub body: Option<String>,
    pub replying_id: Option<i32>,
    pub sent_time: NaiveDateTime,
    pub is_read: Option<bool>,
    pub is_deleted: bool,
}


pub enum MessageInteractionError {
    MessageIsDeleted
}


impl Message {
    pub fn send(conn: &mut SqliteConnection, sender: &db::User, receiver: &db::User, replying_to: Option<&Message>, contents: &str) -> Self {
        diesel::insert_into(messages)
            .values(&Message {
                id: gen_id(),
                sender_id: sender.id,
                receiver_id: receiver.id,
                body: Some(contents.to_string()),
                replying_id: replying_to.map(|msg| msg.id),
                sent_time: Utc::now().naive_local(),
                is_read: Some(false),
                is_deleted: false,
            })
            .get_result(conn)
            .unwrap()
    }
    
    pub fn get(conn: &mut SqliteConnection, id_: i32) -> Option<Self> {
        messages
            .find(id_)
            .get_result(conn)
            .optional()
            .unwrap()
    }
    
    pub fn mark_as_read(&mut self, conn: &mut SqliteConnection) -> Result<(), MessageInteractionError> {
        if self.is_deleted {
            return Err(MessageInteractionError::MessageIsDeleted);
        };
        
        diesel::update(messages.find(self.id))
            .set(is_read.eq(Some(true)))
            .execute(conn)
            .unwrap();
        
        Ok(())
    }
    
    pub fn is_accessible_to(&self, user: &db::User) -> bool {
        self.sender_id == user.id || self.receiver_id == user.id
    }

    pub fn get_body_preview(&self, preview_length: usize) -> Result<&str, MessageInteractionError> {
        Ok(&self.body.as_ref().ok_or(MessageInteractionError::MessageIsDeleted)?
            [..preview_length.min(self.body.as_ref().expect("the body was already checked is deleted").len())])
    }
    
    pub fn get_replying_msg(&self, conn: &mut SqliteConnection) -> Option<Self> {
        self.replying_id.map(|id_| Self::get(conn, id_).unwrap())
    }
    
    pub fn get_all_not_deleted_made_by_user(conn: &mut SqliteConnection, user: &db::User) -> Vec<Self> {
        messages
            .filter(
                sender_id.eq(user.id)
                    .and(is_deleted.eq(false))
            )
            .select(Self::as_select())
            .get_results(conn)
            .unwrap()
    }

    pub fn get_all_not_deleted_accessible_to_user(conn: &mut SqliteConnection, user: &db::User, descending_order: bool, count: i64, offset: u64) -> Vec<Self> {
        let base_stmt = messages
            .filter(
                sender_id.eq(user.id)
                    .or(receiver_id.eq(user.id))
                    .and(is_deleted.eq(false))
            )
            .offset(offset as i64)  // todo return err if it doesnt fit
            .limit(count);


        if descending_order {
            base_stmt
                .order_by(sent_time.desc())
                .select(Self::as_select())
                .get_results(conn)
        } else {
            base_stmt
                .order_by(sent_time.asc())
                .select(Self::as_select())
                .get_results(conn)
        }.unwrap()
    }
    
    pub fn get_all_not_deleted_replies(&self, conn: &mut SqliteConnection) -> Vec<Self> {
        messages
            .filter(replying_id.eq(self.id))
            .get_results(conn)
            .unwrap()
    }

    pub fn delete(&mut self, conn: &mut SqliteConnection) {
        // xxx or should it return UserInteractionError::UserIsDeleted error?
        if self.is_deleted {
            return;
        };
        
        diesel::update(messages.find(self.id))
            .set((
                body.eq(Option::<String>::None),
                is_read.eq(Option::<bool>::None),
                is_deleted.eq(true),
            ))
            .execute(conn)
            .unwrap();
        
        self.body = None;
        self.is_read = None;
        
        self.is_deleted = true;
    }

    pub fn get_sender(&self, conn: &mut SqliteConnection) -> db::User {
        messages.find(self.id)
            .inner_join(users.on(users::dsl::id.eq(self.sender_id)))
            .select(db::User::as_select())
            .get_result(conn)
            .unwrap()
    }

    pub fn get_receiver(&self, conn: &mut SqliteConnection) -> db::User {
        messages.find(self.id)
            .inner_join(users.on(users::dsl::id.eq(self.receiver_id)))
            .select(db::User::as_select())
            .get_result(conn)
            .unwrap()
    }
}
