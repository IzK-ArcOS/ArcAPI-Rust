use diesel::{
    define_sql_function,
    sql_types::*,
    expression::TypedExpressionType
};


define_sql_function! {
    #[sql_name = "JSON_EXTRACT"]
    fn json_extract<T: TypedExpressionType>(s: Text, path: VarChar) -> T; 
}
