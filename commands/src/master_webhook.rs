// pub mod auto;
pub mod manual;

use anyhow::Result;

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
pub struct MasterWebhook {
    pub guild_id: u64,
    pub server_name: String,
    pub webhook_url: String,
    pub public_key_pem: String,
}

impl MasterWebhook {
    pub fn new(guild_id: u64, server_name: &str, webhook_url: &str, public_key_pem: &str) -> Self {
        Self {
            guild_id,
            server_name: server_name.to_string(),
            webhook_url: webhook_url.to_string(),
            public_key_pem: public_key_pem.to_string(),
        }
    }

    pub fn from_row(
        guild_id: &str,
        server_name: &str,
        webhook_url: &str,
        public_key_pem: &str,
    ) -> Result<Self> {
        let guild_id = guild_id.parse::<u64>()?;
        Ok(Self {
            guild_id,
            server_name: server_name.to_string(),
            webhook_url: webhook_url.to_string(),
            public_key_pem: public_key_pem.to_string(),
        })
    }
}
