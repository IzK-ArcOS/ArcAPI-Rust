pub mod users;
pub mod tokens;
pub mod messages;


fn gen_id() -> i32 {
    rand::random::<i32>().abs()
}
