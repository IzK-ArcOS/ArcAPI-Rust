mod db;

fn main() {
    // todo actual logging by using actual logger
    
    println!("loading .env");
    let _ = dotenvy::dotenv().unwrap();
    
    println!("Users in the ArcAPI DB:");
    
    let conn_pool = db::create_db_connection_pool();
    let conn = &mut conn_pool.get().unwrap();
    
    for user in db::User::get_all(conn) {
        println!("#{} - {}", user.id, (!user.is_deleted)
            .then_some(user.username.unwrap().as_str())
            .unwrap_or("[deleted]"));
    };
}
