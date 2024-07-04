use chrono::NaiveDateTime;
use diesel::prelude::*;
use crate::db;
use super::super::schema::{self, messages::dsl::*};


#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Message {
    pub id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub body: Option<String>,
    pub replying_id: i32,
    pub sent_time: NaiveDateTime,
    pub is_read: Option<bool>,
    pub is_deleted: bool,
}


impl Message {
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
}
