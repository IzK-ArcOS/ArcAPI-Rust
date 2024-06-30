use diesel::{
    prelude::*,
    sql_types::Bool
};
use chrono::NaiveDateTime;
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


impl User {
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
            .filter(
                is_deleted.eq(false)
                    .and(functions::json_extract::<Bool, _, _>(properties.assume_not_null(), "$.acc.enabled").eq(true))
            )
            .select(Self::as_select())
            .load(conn)
            .unwrap()
    }
}
