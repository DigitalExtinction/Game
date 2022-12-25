use std::{error::Error, fmt};

pub trait Validatable {
    fn validate(&self) -> Result;
}

pub(crate) type Result = std::result::Result<(), ValidationError>;

#[derive(Debug)]
pub struct ValidationError(String);

impl ValidationError {
    pub(crate) fn new(message: String) -> Self {
        Self(message)
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ValidationError {}

#[macro_export]
macro_rules! ensure {
    ($valid:expr, $fmt:expr) => {
        if !$valid {
            return Err($crate::validation::ValidationError::new(format!($fmt)));
        }
    };
    ($valid:expr, $fmt:expr, $($arg:tt)*) => {
        if !$valid {
            return Err($crate::validation::ValidationError::new(format!($fmt, $($arg)*)));
        }
    };
}
