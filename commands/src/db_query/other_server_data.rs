
use super::*;

use crate::other_server::OtherServerData;

use anyhow::Result;
use sled::Db;
use tracing::error;

pub struct OtherServerDataTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OtherServerDataTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OtherServerDataTable<'a> {
    const TABLE_NAME: &'static str = "OtherServerDataTable";

    type SledKey = String;

    type SledValue = OtherServerData;

    fn get_db(&self) -> &sled::Db {
        self.db
    }
}

impl OtherServerData {
    // server_nameを一意にするために，keyをserver_nameにする
    pub fn db_upsert(&self, db: &Db) -> Result<()> {
        let other_server_table = OtherServerDataTable::new(db);
        other_server_table.upsert(&self.server_name, self)?;
        Ok(())
    }

    pub fn db_read(db: &Db, server_name: &str) -> Result<Option<Self>> {
        let other_server_table = OtherServerDataTable::new(db);
        let data = other_server_table.read(&server_name.to_owned())?;
        Ok(data)
    }

    pub fn db_read_all(db: &Db) -> Result<Vec<Self>> {
        let other_server_table = OtherServerDataTable::new(db);
        let data = other_server_table.read_all()?;
        Ok(data)
    }

    pub fn db_read_from_guild_id(db: &Db, guild_id: u64) -> Result<Self> {
        let other_server_table = OtherServerDataTable::new(db);
        let data = other_server_table.read_all()?;
        // guild_idが一致するものを抽出する
        let mut filtered_data: Vec<Self> = data
            .into_iter()
            .filter(|x| x.guild_id == guild_id)
            .collect();

        // 要素数は1か0になるはずである
        if filtered_data.len() > 1 {
            error!("guild_idが一致するOtherServerDataが2つ以上あります");
        }
        let filtered_data = filtered_data.remove(0);
        
        Ok(filtered_data)
    }

    pub fn db_delete(db: &Db, server_name: &str) -> Result<()> {
        let other_server_table = OtherServerDataTable::new(db);
        other_server_table.delete(&server_name.to_owned())?;
        Ok(())
    }
}
