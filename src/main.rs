mod db;
mod routers;


#[tokio::main]
async fn main() {
    // todo logger
    
    dotenvy::dotenv().expect(".env file should be a valid env vars file");
    
    let conn_pool = db::create_db_connection_pool();
    
    db::migrate(&mut conn_pool.get().unwrap());    
    
    let app = axum::Router::new()
        .nest("/", routers::v1::get_router())
        // .nest("/v2", routers::v2::get_router())  // fixme uncomment once implemented
        .with_state(conn_pool);

    // todo server address config
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
