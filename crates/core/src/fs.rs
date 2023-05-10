use async_std::path::PathBuf;
use thiserror::Error;

/// Returns DE configuration directory.
pub fn conf_dir() -> Result<PathBuf, DirError> {
    let Some(base_conf_dir) = dirs::config_dir() else {
        return Err(
            DirError("User's configuration directory cannot be established.")

        );
    };

    Ok(PathBuf::from(base_conf_dir).join("DigitalExtinction"))
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct DirError(&'static str);
