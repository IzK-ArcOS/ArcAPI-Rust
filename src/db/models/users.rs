use diesel::{
    prelude::*,
    sql_types::Bool
};
use chrono::NaiveDateTime;
use diesel::result::DatabaseErrorKind;
use serde_json::json;
use super::super::{
    schema::{self, users::dsl::*},
    functions
};


#[derive(Queryable, Selectable)]
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

#[derive(Insertable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct NewUser {
    pub username: Option<String>,
    pub hashed_password: Option<String>,
    pub creation_time: NaiveDateTime,
    pub properties: Option<String>,
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
            .values(&NewUser {
                username: Some(username_.to_string()),
                hashed_password: Some(Self::hash_password(password)),
                creation_time: chrono::Utc::now().naive_local(),
                properties: Some(properties_.unwrap_or(&json!({"acc": {}})).to_string()),  // xxx should the default be in a file or a const? or this is fine?
                is_deleted: false,
            })
            .get_result(conn);

        match r {
            Ok(r) => Ok(r),
            Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Err(UserCreationError::SuchUsernameIsAlreadyUsed),
            err @ Err(_) => { err.unwrap(); unreachable!("'err' is an Err variant, so unwrap must fail") }
        }
    }

    pub fn map_properties_as_json(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        self.properties.as_ref().map(|prop_raw| serde_json::from_str(prop_raw))
    }
    
    pub fn get_all(conn: &mut SqliteConnection) -> Vec<Self> {
        users
            .select(Self::as_select())
            .load(conn)
            .unwrap()
    }
    
    pub fn get_all_accessible(conn: &mut SqliteConnection) -> Vec<Self> {
        users
            .filter(is_deleted.eq(false))
            .select(Self::as_select())
            .load(conn)
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
}
