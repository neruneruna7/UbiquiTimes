use thiserror::Error;

use crate::own_server::{OwnServer, OwnTimes};

pub mod sled_own_server_repository;
pub mod sled_own_times_repository;

pub use sled_own_server_repository::SledOwnServerRepository;
pub use sled_own_times_repository::SledOwnTimesRepository;

#[derive(Debug, Error)]
pub enum OwnServerRepositoryError {
    // お試しでthiserrorを導入してみる
    #[error("Database error: {0}")]
    DatabaseError(#[from] sled::Error),
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    // 他のエラータイプもここに追加できます
}

pub type OwnServerResult<T> = Result<T, OwnServerRepositoryError>;

pub trait OwnServerRepository {
    // 結局のところ，複数レコードを扱うことはない
    // つねに1つのレコードしかない
    async fn upsert(&self, own_server: OwnServer) -> OwnServerResult<OwnServer>;
    // レコードがない場合，今回はNoneとせずにエラーとする
    async fn get(&self) -> OwnServerResult<OwnServer>;
    async fn delete(&self) -> OwnServerResult<OwnServer>;
}

pub type OwnTimesRepositoryError = OwnServerRepositoryError;
pub type OwnTimesResult<T> = Result<T, OwnTimesRepositoryError>;

pub trait OwnTimesRepository {
    async fn upsert(&self, own_times: OwnTimes) -> OwnTimesResult<OwnTimes>;
    async fn get(&self, member_id: u64) -> OwnTimesResult<Option<OwnTimes>>;
    async fn get_all(&self) -> OwnTimesResult<Vec<OwnTimes>>;
    async fn delete(&self, member_id: u64) -> OwnTimesResult<Option<OwnTimes>>;
}
