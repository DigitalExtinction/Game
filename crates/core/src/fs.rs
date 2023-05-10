use std::path::PathBuf as SyncPathBuf;

use async_std::path::PathBuf as AsyncPathBuf;
use thiserror::Error;

/// Returns DE configuration directory.
pub fn conf_dir() -> Result<AsyncPathBuf, DirError> {
    dir(dirs::config_dir)
}

/// Returns DE logging directory.
pub fn logs_dir() -> Result<AsyncPathBuf, DirError> {
    dir(dirs::cache_dir).map(|d| d.join("logs"))
}

fn dir<F>(base_dir: F) -> Result<AsyncPathBuf, DirError>
where
    F: Fn() -> Option<SyncPathBuf>,
{
    let Some(base_dir) = base_dir() else {
        return Err(
            DirError("Base directory cannot be established.")
       );
    };

    Ok(AsyncPathBuf::from(base_dir).join("DigitalExtinction"))
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct DirError(&'static str);
