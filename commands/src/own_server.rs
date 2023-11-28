use crate::db_query::SledTable;
use anyhow::Result;
use serde::{Deserialize, Serialize};
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

