use crate::db_query::SledTable;
use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::FromRow;

// pub mod command;
pub mod server;
pub mod times;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, FromRow)]
pub struct OwnServerData {
    pub guild_id: u64,
    pub server_name: String,
    pub master_channel_id: u64,
    pub master_webhook_url: String,
    pub private_key_pem: String,
    pub public_key_pem: String,
}

impl OwnServerData {
    pub fn new(
        guild_id: u64,
        server_name: &str,
        master_channel_id: u64,
        master_webhook_url: &str,
        private_key_pem: &str,
        public_key_pem: &str,
    ) -> Self {
        Self {
            guild_id,
            server_name: server_name.to_string(),
            master_channel_id,
            master_webhook_url: master_webhook_url.to_string(),
            private_key_pem: private_key_pem.to_string(),
            public_key_pem: public_key_pem.to_string(),
        }
    }
}

pub struct OwnServerDataTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OwnServerDataTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OwnServerDataTable<'a> {
    const TABLE_NAME: &'static str = "OwnServerDataTable";
    // const KEY_NAME: &'static str = "OwnServerData";
    // 自身のサーバーデータは常に1つしかない
    // keyにどんな値を渡したとしても同じキーを使うように
    // それぞれにkeyの引数を減らして実装したいが，その方法がわからないのでこの手段をとった
    // クソコードだね

    type SledKey = String;

    type SledValue = OwnServerData;

    fn get_db(&self) -> &sled::Db {
        self.db
    }

    fn upsert(&self, key: &Self::SledKey, value: &Self::SledValue) -> Result<()> {
        let key = "OwnServerData";
        let value = serde_json::to_string(value)?;
        let byte_key = value.as_bytes();
        let db = self.get_db();
        db.open_tree(Self::TABLE_NAME)?.insert(key, byte_key)?;
        Ok(())
    }

    fn read(&self, key: &Self::SledKey) -> Result<Option<Self::SledValue>> {
        let db = self.get_db();
        let byte_key = "OwnServerData";
        let ret = db.open_tree(Self::TABLE_NAME)?.get(byte_key)?;
        match ret {
            Some(ivec) => {
                let string = String::from_utf8(ivec.to_vec())?;
                let value = serde_json::from_str::<Self::SledValue>(&string)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn read_all(&self) -> Result<Vec<Self::SledValue>> {
        let db = self.get_db();
        let mut ret = Vec::new();
        let tree = db.open_tree(Self::TABLE_NAME)?;
        for item in tree.iter() {
            let (key, value) = item?;
            let string = String::from_utf8(value.to_vec())?;
            let value = serde_json::from_str::<Self::SledValue>(&string)?;
            ret.push(value);
        }
        Ok(ret)
    }

    fn delete(&self, key: &Self::SledKey) -> Result<()> {
        let db = self.get_db();
        let byte_key = "OwnServerData";
        db.open_tree(Self::TABLE_NAME)?.remove(byte_key)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, FromRow)]
pub struct OwnTimesData {
    pub member_id: u64,
    pub member_name: String,
    pub channel_id: u64,
    pub webhook_url: String,
}

impl OwnTimesData {
    pub fn new(member_id: u64, member_name: &str, channel_id: u64, webhook_url: &str) -> Self {
        Self {
            member_id,
            member_name: member_name.to_string(),
            channel_id,
            webhook_url: webhook_url.to_string(),
        }
    }
}

pub struct OwnTimesDataTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OwnTimesDataTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OwnTimesDataTable<'a> {
    const TABLE_NAME: &'static str = "OwnTimesDataTable";

    type SledKey = String;

    type SledValue = OwnTimesData;

    fn get_db(&self) -> &sled::Db {
        self.db
    }
}
