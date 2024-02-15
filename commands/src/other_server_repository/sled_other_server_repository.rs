use super::{OtherServer, OtherServerRepository, OtherServerResult};

use crate::sled_table::SledTable;
use tracing::error;

pub struct SeledOtherServerRepository {
    db: sled::Db,
}

impl SeledOtherServerRepository {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }
}

impl OtherServerRepository for SeledOtherServerRepository {
    async fn upsert(&self, other_server: OtherServer) -> OtherServerResult<OtherServer> {
        let other_server_table = OtherServerTable::new(&self.db);
        other_server_table.upsert(&other_server.server_name, &other_server)?;
        Ok(other_server)
    }

    async fn get(&self, server_name: &str) -> OtherServerResult<Option<OtherServer>> {
        let other_server_table = OtherServerTable::new(&self.db);
        let data = other_server_table.read(&server_name.to_owned())?;
        Ok(data)
    }

    async fn get_all(&self) -> OtherServerResult<Vec<OtherServer>> {
        let other_server_table = OtherServerTable::new(&self.db);
        let data = other_server_table.read_all()?;
        Ok(data)
    }

    async fn get_from_guild_id(&self, guild_id: u64) -> OtherServerResult<Option<OtherServer>> {
        let other_server_table = OtherServerTable::new(&self.db);
        let data = other_server_table.read_all()?;
        // guild_idが一致するものを抽出する
        let mut filtered_data: Vec<OtherServer> = data
            .into_iter()
            .filter(|x| x.guild_id == guild_id)
            .collect();

        // 要素数は1か0になるはずである
        if filtered_data.len() > 1 {
            error!("guild_idが一致するOtherServerDataが2つ以上あります");
        }
        if filtered_data.is_empty() {
            return Ok(None);
        }
        let filtered_data = filtered_data.remove(0);

        Ok(Some(filtered_data))
    }

    async fn delete(&self, server_name: &str) -> OtherServerResult<OtherServer> {
        // 雑実装なのでリファクタする方法があるはず
        let other_server_table = OtherServerTable::new(&self.db);
        // 一度取得するのは絶対に無駄 sledtableトレイトを修正するべきだろう
        let data = other_server_table.read(&server_name.to_owned())?;
        other_server_table.delete(&server_name.to_owned())?;
        Ok(data.unwrap())
    }
}

struct OtherServerTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OtherServerTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OtherServerTable<'a> {
    const TABLE_NAME: &'static str = "OtherServerDataTable";

    type SledKey = String;

    type SledValue = OtherServer;

    fn get_db(&self) -> &sled::Db {
        self.db
    }
}
