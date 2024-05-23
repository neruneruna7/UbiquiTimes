use super::other_times_repository::SledOtherTimesRepositoryError;
use super::own_guild_repository::SledOwnGuildRepositoryError;
use super::own_times_repository::SledOwnTimesRepositoryError;
use domain::thiserror;

#[derive(thiserror::Error, Debug)]
pub enum RepositoryError {
    #[error("OwnGuildRepositoryError: {0}")]
    OwnGuildRepositoryError(#[from] SledOwnGuildRepositoryError),
    #[error("OtherTimesRepositoryError: {0}")]
    OtherTimesRepositoryError(#[from] SledOtherTimesRepositoryError),
    #[error("OwnTimesRepositoryError: {0}")]
    OwnTimesRepositoryError(#[from] SledOwnTimesRepositoryError),
}
