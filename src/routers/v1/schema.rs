use serde::{Deserialize, Serialize};
use crate::db;


#[derive(Debug, Serialize, Deserialize)]
pub struct DataResponse<T> {
    data: T,
    valid: bool
}


impl<T> DataResponse<T> {
    pub fn new(data: T) -> Self {
        Self { valid: true, data }
    }
    
    pub fn get_data(self) -> T {
        self.data
    }
    
    pub fn get_data_ref(&self) -> &T {
        &self.data
    }
    
    pub fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PartialUser {
    username: Box<str>,
    acc: serde_json::Value
}


pub enum ConversionError {
    ItemIsDeleted, ItemIsCorrupted(bool)
}



impl TryFrom<db::User> for PartialUser {
    type Error = ConversionError;
    
    fn try_from(u: db::User) -> Result<Self, Self::Error> {
        if u.is_deleted {
            return Err(ConversionError::ItemIsDeleted);
        };
        
        match u.get_properties_as_json().expect("user is not deleted") {
            Err(_) => Err(ConversionError::ItemIsCorrupted(false)),
            Ok(mut json_prop) =>
                Ok(Self {
                    username: u.username.unwrap().into(),
                    acc: json_prop.get_mut("acc")
                        .ok_or(ConversionError::ItemIsCorrupted(true))?
                        .take()
                })
        }
    }
}
