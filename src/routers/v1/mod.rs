use crate::AppState;

mod users;
mod schema;
mod user;
mod meta;


pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .nest("/users", users::get_router())
        .nest("/user", user::get_router())
        .nest("/connect", meta::get_router())
}
