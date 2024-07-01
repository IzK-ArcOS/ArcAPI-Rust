mod db;
mod routers;
mod config;


use std::sync::Arc;
use config::Config;


#[derive(Debug, Clone)]
struct AppState {
    pub conn_pool: db::ConnPool,
    pub config: Arc<Config>,
}


#[tokio::main]
async fn main() {
    // todo logger
    
    dotenvy::dotenv().expect(".env file should be a valid env vars file");
    
    let config = Config::load();
    
    let conn_pool = db::create_db_connection_pool(&config.database.path, config.database.conn_pool_size);
    
    db::migrate(&mut conn_pool.get().unwrap());
    
    // todo remove this to string and then later from string conversion, while still supporting V4 and V6
    let addr = format!("{}:{}", config.server.address, config.server.port);
    
    let app = axum::Router::new()
        .nest("/", routers::v1::get_router())
        // .nest("/v2", routers::v2::get_router())  // fixme uncomment once implemented
        .with_state(AppState { conn_pool, config: Arc::new(config) });
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
