#[macro_export]
macro_rules! log_full_error {
    ($err:ident) => {
        use std::error::Error;
        use std::fmt::Write;

        use bevy::prelude::error;

        let mut error_message = format!("{}", $err);
        let mut error: &dyn Error = &$err;
        while let Some(source) = error.source() {
            error = source;
            write!(&mut error_message, ": {}", error).unwrap();
        }
        error!("{}", error_message);
    };
}
