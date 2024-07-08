use serde::{Deserialize, Serialize};
use crate::db;

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialUser {
    pub username: String,
    pub acc: serde_json::Value
}


pub enum ConversionError {
    ItemIsDeleted, ItemIsCorrupted(bool)
}


impl PartialUser {
    pub fn try_new(u: &db::User) -> Result<Self, ConversionError> {
        if u.is_deleted {
            return Err(ConversionError::ItemIsDeleted);
        };

        match u.map_properties_as_json().expect("user is not deleted") {
            Err(_) => Err(ConversionError::ItemIsCorrupted(false)),
            Ok(mut json_prop) =>
                Ok(Self {
                    username: u.username.as_ref().unwrap().to_string(),
                    acc: json_prop.get_mut("acc")
                        .ok_or(ConversionError::ItemIsCorrupted(true))?
                        .take()
                })
        }
    } 
}
