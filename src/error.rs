use std::io;

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("Update failed! Stop the timer first")]
    UpdateError(),

    #[error("There was an error with saving your data: {0}")]
    SaveError(String),

    #[error("There was an error with loading your data: {0}")]
    LoadError(String),

    #[error("Couldn't locate a suitable directory to keep your config in.")]
    HomeDirNotFound,

    #[error("Error with filesystem: {0}")]
    IO(#[from] io::Error),
}
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] io::Error),

    #[error("Settings Error: {0}")]
    SettingsError(#[from] SettingsError),
    
}
pub type Result<T> = std::result::Result<T,Error>;
