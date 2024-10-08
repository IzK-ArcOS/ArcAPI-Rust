mod db;
mod routers;
mod config;
mod middleware;
mod filesystem;
mod env;


use std::sync::Arc;
use axum::extract::Request;
use axum::ServiceExt;
use tower::Layer;
use config::Config;
use filesystem::Filesystem;
use crate::env::load_dotenv;


#[derive(Debug, Clone)]
struct AppState {
    pub conn_pool: db::ConnPool,
    pub config: Arc<Config>,
    pub filesystem: Arc<Filesystem>,
}


#[tokio::main]
async fn main() {
    // xxx should logging level be configurable?
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    log::info!("starting up...");
    
    load_dotenv();
    
    let config = Config::load();
    
    let conn_pool = db::create_db_connection_pool(&config.database.path, config.database.conn_pool_size);
    
    db::migrate(&mut conn_pool.get().unwrap());
    
    let filesystem = Filesystem::new(
        &config.filesystem.storage_path, 
        config.filesystem.template_path.as_deref(),
        config.filesystem.total_size,
        config.filesystem.user_space_size
    );
    
    // todo remove this to string and then later from string conversion, while still supporting V4 and V6
    let addr = format!("{}:{}", config.server.address, config.server.port);
    
    let state = AppState { conn_pool, filesystem: Arc::new(filesystem), config: Arc::new(config) };

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

    log::info!("starting server!!!");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await.unwrap();
}
