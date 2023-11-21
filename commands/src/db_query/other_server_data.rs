use super::*;

use crate::other_server::{OtherServerData, OtherServerDataTable};

use sled::Db;
use anyhow::{Result};

impl OtherServerData {
    // server_nameを一意にするために，keyをserver_nameにする
    pub fn db_upsert(&self, db: &Db) -> Result<()> {
        let other_server_table = OtherServerDataTable::new(db);
        let _ = other_server_table.upsert(&self.server_name, self)?;
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

    pub fn db_read_from_guild_id(db: &Db, guild_id: u64) -> Result<Vec<Self>> {
        let other_server_table = OtherServerDataTable::new(db);
        let data = other_server_table.read_all()?;
        // guild_idが一致するものを抽出する
        let filtered_data: Vec<Self> = data.into_iter().filter(|x| x.guild_id == guild_id).collect();
        Ok(filtered_data)
    }

    pub fn db_delete(db: &Db, server_name: &str) -> Result<()> {
        let other_server_table = OtherServerDataTable::new(db);
        let _ = other_server_table.delete(&server_name.to_owned())?;
        Ok(())
    }
}