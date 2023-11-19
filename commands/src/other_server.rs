// pub mod auto;
// pub mod manual;

pub mod server;
pub mod times;

use anyhow::Result;
use crate::db_query::SledTable;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// 相手サーバーのデータ
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct OtherServerData {
    pub guild_id: u64,
    pub server_name: String,
    pub webhook_url: String,
    pub public_key_pem: String,
}

impl OtherServerData {
    pub fn new(guild_id: u64, server_name: &str, webhook_url: &str, public_key_pem: &str) -> Self {
        Self {
            guild_id,
            server_name: server_name.to_string(),
            webhook_url: webhook_url.to_string(),
            public_key_pem: public_key_pem.to_string(),
        }
    }
}

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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
// 他サーバのtimesデータ
pub struct OtherTimesData {
    pub src_member_id: u64,
    pub dst_server_name: String,
    pub dst_guild_id: u64,
    pub dst_channel_id: u64,
    pub dst_webhook_url: String,
}

impl OtherTimesData {
    pub fn new(
        src_member_id: u64,
        dst_server_name: &str,
        dst_guild_id: u64,
        dst_channel_id: u64,
        dst_webhook_url: &str,
    ) -> Self {
        Self {
            src_member_id,
            dst_server_name: dst_server_name.to_string(),
            dst_guild_id,
            dst_channel_id,
            dst_webhook_url: dst_webhook_url.to_string(),
        }
    }
}

pub struct OtherTimesDataTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OtherTimesDataTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}


impl<'a> SledTable for OtherTimesDataTable<'a> {
    const TABLE_NAME: &'static str = "OtherTimesDataTable";

    type SledKey = String;

    type SledValue = OtherTimesData;

    fn get_db(&self) -> &sled::Db {
        self.db
    }
}
