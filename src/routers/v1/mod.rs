use crate::db;

mod users;
mod schema;


pub fn get_router() -> axum::Router<db::ConnPool> {
    axum::Router::new()
        .nest("/users", users::get_router())
}
