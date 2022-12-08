pub const SQLITE_CONSTRAINT_PRIMARYKEY: &str = "1555";
pub const SQLITE_CONSTRAINT_UNIQUE: &str = "2067";

#[macro_export]
macro_rules! db_error {
    ($result:expr, $error:expr, $code:expr) => {
        if let Err(sqlx::Error::Database(ref error)) = $result {
            if let Some(code) = error.code() {
                if code == $code {
                    return Err($error);
                }
            }
        }
    };
}
