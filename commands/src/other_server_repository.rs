use crate::other_server::{OtherServer, OtherTimes};
use anyhow::Result;

pub mod sled_other_server_repository;
pub mod sled_other_times_repository;

pub use sled_other_server_repository::SledOtherServerRepository;
pub use sled_other_times_repository::SledOtherTimesRepository;

pub type OtherServerResult<T> = Result<T>;
pub type OtherTimesResult<T> = Result<T>;

pub trait OtherServerRepository {
    // 正直asyncはいらない
    async fn upsert(&self, other_server: OtherServer) -> OtherServerResult<OtherServer>;
    async fn get(&self, server_name: &str) -> OtherServerResult<Option<OtherServer>>;
    async fn get_all(&self) -> OtherServerResult<Vec<OtherServer>>;
    async fn get_from_guild_id(&self, guild_id: u64) -> OtherServerResult<Option<OtherServer>>;
    async fn delete(&self, server_name: &str) -> OtherServerResult<OtherServer>;
}

pub trait OtherTimesRepository {
    async fn upsert(&self, other_times: OtherTimes) -> OtherTimesResult<OtherTimes>;
    async fn get(&self, server_name: &str, member_id: u64) -> OtherTimesResult<Option<OtherTimes>>;
    async fn get_all(&self) -> OtherTimesResult<Vec<OtherTimes>>;
    async fn get_from_member_id(&self, member_id: u64) -> OtherTimesResult<Vec<OtherTimes>>;
    async fn delete(
        &self,
        server_name: &str,
        member_id: u64,
    ) -> OtherTimesResult<Option<OtherTimes>>;
}
