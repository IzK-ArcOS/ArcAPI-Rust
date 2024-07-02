use axum_typed_multipart::TryFromMultipart;
use serde::{Deserialize, Serialize};
use crate::db;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub access_token: String
}


#[derive(Serialize, Deserialize, TryFromMultipart)]
pub struct NewSession {
    pub username: String,
    pub password: String,
}



impl From<db::Token> for Session {
    fn from(t: db::Token) -> Self {
        Self { access_token: t.value }
    }
}
