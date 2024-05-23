// 使用されなくなっていたため，整理のためGabageへ移動

mod domain {
    // もともとdomainのtralts内にトレイトがあったが，こちらも整理のため移動
    // 区別のためにmodを使用
    // 実際今のような配置だったわけではない
    pub trait OtherGuildRepository {
        // 正直asyncはいらない
        type Result<T>;

        fn upsert(
            &self,
            other_server: OtherGuild,
        ) -> impl std::future::Future<Output = Self::Result<OtherGuild>> + Send;
        fn get(
            &self,
            guild_id: u64,
        ) -> impl std::future::Future<Output = Self::Result<Option<OtherGuild>>> + Send;
        fn get_from_guildname(
            &self,
            server_name: &str,
        ) -> impl std::future::Future<Output = Self::Result<Option<OtherGuild>>> + Send;
        fn get_all(&self) -> impl std::future::Future<Output = Self::Result<Vec<OtherGuild>>> + Send;
        fn delete(
            &self,
            server_name: &str,
        ) -> impl std::future::Future<Output = Self::Result<OtherGuild>> + Send;
    }
}

use crate::sled_table::{SledTable, SledTableError};
use domain::models::guild_data::OtherGuild;
use domain::traits::repositorys::OtherGuildRepository;
use domain::{thiserror, tracing};
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum SledOtherGuildRepositoryError {
    #[error("SledError: {0}")]
    SledError(#[from] sled::Error),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("StringError: {0}")]
    StringError(#[from] std::string::FromUtf8Error),
    #[error("SledTableError: {0}")]
    SledTableError(#[from] SledTableError),
}

pub type SledOtherGuildRepositoryResult<T> = Result<T, SledOtherGuildRepositoryError>;

#[derive(Debug)]
pub struct SledOtherGuildRepository {
    db: sled::Db,
}

impl SledOtherGuildRepository {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }
}

impl OtherGuildRepository for SledOtherGuildRepository {
    type Result<T> = SledOtherGuildRepositoryResult<T>;
    fn upsert(&self, other_server: OtherGuild) -> Self::Result<OtherGuild> {
        let other_server_table = OtherGuildTable::new(&self.db);
        other_server_table.upsert(&other_server.guild_name, &other_server)?;
        Ok(other_server)
    }

    fn get(&self, server_name: &str) -> Self::Result<Option<OtherGuild>> {
        let other_server_table = OtherGuildTable::new(&self.db);
        let data = other_server_table.read(&server_name.to_owned())?;
        Ok(data)
    }

    fn get_all(&self) -> Self::Result<Vec<OtherGuild>> {
        let other_server_table = OtherGuildTable::new(&self.db);
        let data = other_server_table.read_all()?;
        Ok(data)
    }

    fn get_from_guild_id(&self, guild_id: u64) -> Self::Result<Option<OtherGuild>> {
        let other_server_table = OtherGuildTable::new(&self.db);
        let data = other_server_table.read_all()?;
        // guild_idが一致するものを抽出する
        let mut filtered_data: Vec<OtherGuild> = data
            .into_iter()
            .filter(|x| x.guild_id == guild_id)
            .collect();

        // 要素数は1か0になるはずである
        if filtered_data.len() > 1 {
            error!("There are more than one OtherGuildData with the same guild_id");
        }
        if filtered_data.is_empty() {
            return Ok(None);
        }
        let filtered_data = filtered_data.remove(0);

        Ok(Some(filtered_data))
    }

    fn delete(&self, server_name: &str) -> Self::Result<OtherGuild> {
        // 雑実装なのでリファクタする方法があるはず
        let other_server_table = OtherGuildTable::new(&self.db);
        // 一度取得するのは絶対に無駄 sledtableトレイトを修正するべきだろう
        let data = other_server_table.read(&server_name.to_owned())?;
        other_server_table.delete(&server_name.to_owned())?;
        Ok(data.unwrap())
    }
}

struct OtherGuildTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OtherGuildTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OtherGuildTable<'a> {
    const TABLE_NAME: &'static str = "OtherGuildDataTable";

    type SledKey = String;

    type SledValue = OtherGuild;

    fn get_db(&self) -> &sled::Db {
        self.db
    }
}
