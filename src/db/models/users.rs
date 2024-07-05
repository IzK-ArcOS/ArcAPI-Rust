// todo move the enabled property to db, and in responses just fetch that value from db
use diesel::{
    prelude::*
};
use chrono::NaiveDateTime;
use diesel::result::DatabaseErrorKind;
use crate::db;
use super::gen_id;
use super::super::schema::{self, users::dsl::*};


#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub username: Option<String>,
    pub hashed_password: Option<String>,
    pub creation_time: NaiveDateTime,
    pub properties: Option<String>,  // todo use serde json
    pub is_deleted: bool
}


#[derive(Debug)]
pub enum UserInteractionError {
    UserIsDeleted
}


#[derive(Debug)]
pub enum UserCreationError {
    SuchUsernameIsAlreadyUsed
}


impl User {
    pub(super) fn hash_password(password: &str) -> String {
        hmac_sha512::Hash::hash(password).map(|b| format!("{b:0>2x}")).concat()
    }

    pub fn create(conn: &mut SqliteConnection, username_: &str, password: &str, properties_: Option<&serde_json::Value>) -> Result<Self, UserCreationError> {
        let r = diesel::insert_into(users)
            .values(&User {
                id: gen_id(),
                username: Some(username_.to_string()),
                hashed_password: Some(Self::hash_password(password)),
                creation_time: chrono::Utc::now().naive_local(),
                properties: Some(properties_.map(|p| p.to_string()).unwrap_or(include_str!("../../../assets/user_properties.default.json").into())),
                is_deleted: false,
            })
            .get_result(conn);

        match r {
            Ok(r) => Ok(r),
            Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Err(UserCreationError::SuchUsernameIsAlreadyUsed),
            err @ Err(_) => { err.unwrap(); unreachable!("'err' is an Err variant, so unwrap must fail") }
        }
    }
    
    pub fn delete(&mut self, conn: &mut SqliteConnection) {
        // xxx or should it return UserInteractionError::UserIsDeleted error?
        if self.is_deleted {
            return;
        };
        
        // tokens will be deleted by a cascade
        
        // ...but messages are not going to delete themselves
        for mut message in db::Message::get_all_not_deleted_made_by_user(conn, self) {
            message.delete(conn);
        };
        
        // ...then delete the user
        diesel::update(users.find(self.id))
            .set((
                username.eq(Option::<String>::None),
                hashed_password.eq(Option::<String>::None),
                properties.eq(Option::<String>::None),
                is_deleted.eq(true)
            ))
            .execute(conn)
            .unwrap();
        
        // ...and sync the model
        self.username = None;
        self.hashed_password = None;
        self.properties = None;
        self.is_deleted = true;
    }
    
    pub fn get_by_username(conn: &mut SqliteConnection, username_: &str) -> Option<Self> {
        users
            .filter(username.eq(username_))
            .get_result(conn)
            .optional()
            .unwrap()
    }

    pub fn map_properties_as_json(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        self.properties.as_ref().map(|prop_raw| serde_json::from_str(prop_raw))
    }
    
    pub fn get_username(&self) -> String {
        self.username.as_ref().map(String::clone).unwrap_or_else(|| format!("deleted#{}", self.id))
    }
    
    pub fn get_all(conn: &mut SqliteConnection) -> Vec<Self> {
        users
            .select(Self::as_select())
            .get_results(conn)
            .unwrap()
    }
    
    pub fn get_all_accessible(conn: &mut SqliteConnection) -> Vec<Self> {
        users
            .filter(is_deleted.eq(false))
            .select(Self::as_select())
            .get_results(conn)
            .unwrap()
    }
    
    pub fn set_properties(&mut self, conn: &mut SqliteConnection, new_prop: serde_json::Value) -> Result<(), UserInteractionError> {
        if let Some(ref mut prop) = self.properties {
            let json_new_prop = new_prop.to_string();

            diesel::update(users.find(self.id))
                .set(properties.eq(&json_new_prop))
                .execute(conn)
                .unwrap();
            
            *prop = json_new_prop;
            
            Ok(())
        } else {
            Err(UserInteractionError::UserIsDeleted)
        }
    }
    
    pub fn rename(&mut self, conn: &mut SqliteConnection, new_name: String) -> Result<(), UserInteractionError> {
        if let Some(ref mut username_) = self.username {
            diesel::update(users.find(self.id))
                .set(username.eq(&new_name))
                .execute(conn)
                .unwrap();

            *username_ = new_name;

            Ok(())
        } else {
            Err(UserInteractionError::UserIsDeleted)
        }
    }
    
    pub fn set_password(&mut self, conn: &mut SqliteConnection, new_password: &str) -> Result<(), UserInteractionError> {
        if let Some(ref mut hashed_password_) = self.hashed_password {
            let new_hashed = Self::hash_password(new_password);

            diesel::update(users.find(self.id))
                .set(hashed_password.eq(&new_hashed))
                .execute(conn)
                .unwrap();
            
            *hashed_password_ = new_hashed;
            
            Ok(())
        } else {
            Err(UserInteractionError::UserIsDeleted)
        }
    }
}
