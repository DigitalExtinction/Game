use sqlx::postgres::PgRow;

// TODO fix thise
pub const SQLITE_CONSTRAINT_PRIMARYKEY: &str = "1555";
pub const SQLITE_CONSTRAINT_FOREIGNKEY: &str = "787";

#[macro_export]
macro_rules! db_error_code {
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

#[macro_export]
macro_rules! db_error_message {
    ($result:expr, $error:expr, $message:expr) => {
        if let Err(sqlx::Error::Database(ref error)) = $result {
            if error.message() == $message {
                return Err($error);
            }
        }
    };
}

pub(crate) trait FromRow
where
    Self: Sized,
{
    type Error;

    fn try_from_row(row: PgRow) -> Result<Self, Self::Error>;
}
