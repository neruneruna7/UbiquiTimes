use crate::other_server::OtherServer;
use anyhow::Result;

mod sled_other_server_Repository;
pub mod sled_other_server_repository;

pub type OtherServerResult<T> = Result<T>;

pub enum OtherServerRepositoryEnum {
    Server(sled_other_server_repository::SledOtherServerRepository),
}

pub trait OtherServerRepository {
    async fn upsert(&self, other_server: OtherServer) -> OtherServerResult<OtherServer>;
    async fn get(&self, server_name: &str) -> OtherServerResult<Option<OtherServer>>;
    async fn get_all(&self) -> OtherServerResult<Vec<OtherServer>>;
    async fn get_from_guild_id(&self, guild_id: u64) -> OtherServerResult<Option<OtherServer>>;
    async fn delete(&self, server_name: &str) -> OtherServerResult<OtherServer>;
}
