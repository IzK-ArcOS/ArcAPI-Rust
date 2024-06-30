mod db;

use axum::{
    routing, http::StatusCode, 
    extract::{State},
    Json,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file should a valid env vars file");
    
    let conn_pool = db::create_db_connection_pool();
    
    let app = axum::Router::new()
        .route("/users", routing::get(get_all_users))
        .with_state(conn_pool);

    // todo server address config
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


/// some schemas
mod sc {
    use chrono::NaiveDateTime;
    use serde::Serialize;

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct User {
        pub id: i32,
        pub username: Option<String>,
        pub creation_time: NaiveDateTime,
        pub properties: Option<serde_json::Value>,
    }
    
    impl From<crate::db::User> for User {
        fn from(u: crate::db::User) -> Self {
            Self { id: u.id, username: u.username, creation_time: u.creation_time, properties: u.properties.map(|s| serde_json::from_str(&s).unwrap()) }
        }
    }
}


async fn get_all_users(
    State(conn_pool): State<db::ConnPool>
) -> (StatusCode, Json<Vec<sc::User>>) {
    let conn = &mut conn_pool.get().unwrap();
    
    let users = db::User::get_all(conn)
        .into_iter().map(sc::User::from).collect::<Vec<_>>();

    (StatusCode::OK, Json(users))
}
