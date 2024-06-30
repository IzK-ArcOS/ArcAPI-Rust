use std::ops::Add;
use std::time::Duration;
use chrono::{NaiveDateTime, TimeDelta, Utc};
use diesel::prelude::*;
use crate::db;
use super::super::schema::{self, tokens::dsl::*, users::dsl::*};


#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::tokens)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Token {
    pub value: String,
    pub owner_id: i32,
    pub lifetime: Option<f32>,
    pub creation_time: NaiveDateTime
}


impl Token {
    pub fn get(conn: &mut SqliteConnection, v: &str) -> Option<Self> {
        tokens
            .find(v)
            .select(Self::as_select())
            .first(conn)
            .optional()
            .unwrap()
    }

    pub fn get_owner(&self, conn: &mut SqliteConnection) -> db::User {
        users
            .find(self.owner_id)
            .select(db::User::as_select())
            .first(conn)
            .unwrap()
    }

    pub fn is_expired(&self) -> bool {
        if let Some(lifetime_) = self.lifetime {
            if self.creation_time.add(TimeDelta::from_std(Duration::from_secs_f32(lifetime_)).unwrap()) < Utc::now().naive_local() {
                return true;
            };
        };
        
        false
    }
    
    pub fn is_valid(&self) -> bool {
        self.is_expired()
    }
    
    pub fn delete(self, conn: &mut SqliteConnection) {
        diesel::delete(tokens.find(&self.value)).execute(conn).unwrap();
    }
}
