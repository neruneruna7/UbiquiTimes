use crate::other_server_repository::OtherServerRepositoryError;
use crate::own_server_repository::OwnServerRepositoryError;
use thiserror::Error;

pub mod setting_commands;
pub mod spreading_commands;

#[derive(Debug, Error)]
pub enum CommandsError {
    #[error("AnyhowError: {0}")]
    AnyhowError(#[from] anyhow::Error),
    #[error("OwnServerRepositoryError: {0}")]
    OwnServerRepositoryError(#[from] OwnServerRepositoryError),
    #[error("OtherServerRepositoryError: {0}")]
    OtherServerRepositoryError(#[from] OtherServerRepositoryError),
}

pub type CommandsResult<T> = Result<T, CommandsError>;
