mod meta;
mod schema;
mod token;
mod users;

use crate::AppState;

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .nest("/token", token::get_router())
        .nest("/users", users::get_router())
        .nest("/", meta::get_router())
}
