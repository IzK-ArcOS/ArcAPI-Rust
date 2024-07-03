use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub properties: Option<serde_json::Value>,
}


#[derive(Serialize, Deserialize)]
pub struct SelfUser {
    pub username: String,
    pub id: i32,
    pub properties: serde_json::Value,
}
