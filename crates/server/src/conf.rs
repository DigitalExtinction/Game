use std::{
    env::{self, VarError},
    str::FromStr,
};

use anyhow::{anyhow, Context, Error, Result};

/// Load and parse the value of an environment variable.
///
/// # Arguments
///
/// * `name` - name of the environment variable to load.
pub fn mandatory<T>(name: &str) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    var(name, None)
}

/// Load and parse the value of an environment variable.
///
/// # Arguments
///
/// * `name` - name of the environment variable to load.
///
/// * `default` - default value to be returned if the environment variable is
///   not set.
pub fn optional<T>(name: &str, default: T) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    var(name, Some(default))
}

/// Load and parse the value of an environment variable.
///
/// # Arguments
///
/// * `name` - name of the environment variable to load.
///
/// * `default` - default value to use if the environment variable is not set.
///   An error is returned if both the env variable is not set and the default
///   value is None.
fn var<T>(name: &str, default: Option<T>) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    match env::var(name) {
        Ok(value) => T::from_str(value.as_str())
            .with_context(|| format!("Failed to parse environment variable \"{}\"", name)),
        Err(VarError::NotPresent) => match default {
            Some(value) => Ok(value),
            None => Err(anyhow!(format!(
                "Mandatory environment variable \"{}\" is not set.",
                name
            ))),
        },
        Err(error) => {
            Err(Error::new(error)
                .context(format!("Failed to load environment variable \"{}\"", name)))
        }
    }
}
