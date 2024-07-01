mod schema;
mod functions;
mod models;


pub use models::users::User;
pub use models::tokens::Token;
pub use models::messages::Message;


use diesel::sqlite::SqliteConnection;
use std::env;
use diesel::connection::SimpleConnection;
use diesel::r2d2::{Pool, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};


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


pub type ConnPool = Pool<ConnectionManager<SqliteConnection>>;


pub fn create_db_connection_pool(database_url: &str, max_pool_size: u32) -> ConnPool {
    Pool::builder()
        .max_size(max_pool_size)
        .connection_customizer(Box::new(CustomConnectionOptions()))
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .unwrap()
}


pub fn migrate(conn: &mut SqliteConnection) {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("migrations should be valid");
}
