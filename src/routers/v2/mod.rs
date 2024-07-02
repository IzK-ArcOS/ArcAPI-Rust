mod meta;
mod schema;

use crate::AppState;

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .nest("/", meta::get_router())
}
