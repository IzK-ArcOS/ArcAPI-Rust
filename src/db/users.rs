use diesel::prelude::*;
use super::schema::users::dsl::*;
use chrono::NaiveDateTime;


#[derive(Queryable, Selectable)]
#[diesel(table_name = super::schema::users)]
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
    pub fn get_all(conn: &mut SqliteConnection) -> Vec<Self> {
        users
            .select(User::as_select())
            .load(conn)
            .unwrap()
    }
}
