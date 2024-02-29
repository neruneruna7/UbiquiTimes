use crate::global_data::Context;
use crate::other_server_repository::OtherServerRepositoryError;
use crate::own_server_repository::OwnServerRepositoryError;
use anyhow::{Context as _, Result};
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

async fn command_check(ctx: &Context<'_>, enter_str: &str, sign_str: &str) -> Result<()> {
    let err_text = format!("{}と入力してください", sign_str);
    if enter_str != sign_str {
        ctx.say(&err_text).await?;
        return Err(anyhow::anyhow!(err_text));
    }

    Ok(())
}
