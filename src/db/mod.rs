pub mod schema;
pub mod users;
pub mod tokens;
pub mod messages;


pub use users::User;


use diesel::sqlite::SqliteConnection;
use std::env;
use diesel::connection::SimpleConnection;
use diesel::r2d2::{Pool, ConnectionManager};


// *technically* adapted from https://stackoverflow.com/a/57717533
#[derive(Debug)]
pub struct CustomConnectionOptions();

impl r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for CustomConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        conn.batch_execute(r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
        "#).map_err(diesel::r2d2::Error::QueryError)
    }
}


const DEFAULT_MAX_CONN_POOL_SIZE: u32 = 16;


pub type ConnPool = Pool<ConnectionManager<SqliteConnection>>;


pub fn create_db_connection_pool() -> ConnPool {
    let database_url = env::var("DATABASE_URL").expect("env var 'DATABASE_URL' should be set");
    let max_pool_size = env::var("DATABASE_MAX_CONN_POOL_SIZE")
        .map_or(
            DEFAULT_MAX_CONN_POOL_SIZE, 
            |s| s.parse()
                .expect("env var 'DATABASE_MAX_CONN_POOL_SIZE' should contain a valid 32-bit unsigned integer")
        );
    
    Pool::builder()
        .max_size(max_pool_size)
        .connection_customizer(Box::new(CustomConnectionOptions()))
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .unwrap()
}
