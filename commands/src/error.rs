use domain::thiserror;
use sled_repository::{
    other_guild_repository::SledOtherGuildRepositoryError,
    other_times_repository::SledOtherTimesRepositoryError,
    own_guild_repository::SledOwnGuildRepositoryError,
    own_times_repository::SledOwnTimesRepositoryError,
};

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    // serenity
    #[error("Serenity error: {0}")]
    Serenity(#[from] poise::serenity_prelude::Error),
    #[error("OwnGuildRepositoryError: {0}")]
    OwnGuildRepositoryError(#[from] SledOwnGuildRepositoryError),
    #[error("OwnTiemsRepositoryError: {0}")]
    OwnTimesRepositoryError(#[from] SledOwnTimesRepositoryError),
    #[error("OtherGuildRepositoryError: {0}")]
    OtherGuildRepositoryError(#[from] SledOtherGuildRepositoryError),
    #[error("OtherTimesRepositoryError: {0}")]
    OtherTimesRepositoryError(#[from] SledOtherTimesRepositoryError),

    #[error("GuildId cannot get")]
    GuildIdCannotGet(#[from] GuildIdCannotGet),
    #[error("OwnTimes not found")]
    OwnTimesNotFound(#[from] OwnTimesNotFound),
}

// GuildIDが取得できないエラー
#[derive(Debug, thiserror::Error)]
pub struct GuildIdCannotGet;

impl std::fmt::Display for GuildIdCannotGet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GuildId cannot get")
    }
}

#[derive(Debug, thiserror::Error)]
pub struct OwnTimesNotFound;

impl std::fmt::Display for OwnTimesNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OwnTimes not found")
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
