mod db;
mod routers;
mod config;
mod middleware;
mod filesystem;


use std::sync::Arc;
use axum::extract::Request;
use axum::ServiceExt;
use tower::Layer;
use config::Config;


#[derive(Debug, Clone)]
struct AppState {
    pub conn_pool: db::ConnPool,
    pub config: Arc<Config>,
}


#[tokio::main]
async fn main() {
    // todo start up logging

    dotenvy::dotenv().expect(".env file should be a valid env vars file");
    
    let config = Config::load();
    
    let conn_pool = db::create_db_connection_pool(&config.database.path, config.database.conn_pool_size);
    
    db::migrate(&mut conn_pool.get().unwrap());
    
    // todo remove this to string and then later from string conversion, while still supporting V4 and V6
    let addr = format!("{}:{}", config.server.address, config.server.port);
    
    let state = AppState { conn_pool, config: Arc::new(config) };

    let router = axum::Router::new()
        .nest("/v2", routers::v2::get_router())
        .nest("/", routers::v1::get_router())
        .with_state(state.clone())
        .route_layer(
            tower::ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(state, middleware::verify_auth_code))  // xxx should it be loaded only if ac is Some?
                .layer(tower_http::cors::CorsLayer::permissive())
        )
        .layer(
            tower::ServiceBuilder::new()
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(tower_http::catch_panic::CatchPanicLayer::new())
        );

    let app = tower_http::normalize_path::NormalizePathLayer::trim_trailing_slash().layer(router);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await.unwrap();
}
