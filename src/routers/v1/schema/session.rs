use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use crate::db;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub username: String,
    pub token: String,
}


impl Session {
    pub fn new(conn: &mut SqliteConnection, token: db::Token) -> Self {
        Self {
            username: token.get_owner(conn).username.expect("token should be valid, so the user shouldn't be deleted"),
            token: token.value
        }
    }
}
