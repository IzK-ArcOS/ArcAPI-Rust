use crate::AppState;

mod users;
mod schema;
mod user;
mod meta;
mod session;
mod messages;
mod utils;
mod filesystem;


pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .nest("/users", users::get_router())
        .nest("/user", user::get_router())
        .nest("/connect", meta::get_router())
        .nest("/messages", messages::get_router())
        .nest("/fs", filesystem::get_router())
        .nest("/", session::get_router())
}
