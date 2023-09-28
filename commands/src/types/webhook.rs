use anyhow::Result;

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
pub struct MasterWebhook {
    pub guild_id: u64,
    pub server_name: String,
    pub webhook_url: String,
    pub private_key_pem: String,
}

impl MasterWebhook {
    pub fn new(guild_id: u64, server_name: &str, webhook_url: &str, public_key_pem: &str) -> Self {
        Self {
            guild_id,
            server_name: server_name.to_string(),
            webhook_url: webhook_url.to_string(),
            private_key_pem: public_key_pem.to_string(),
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
            private_key_pem: public_key_pem.to_string(),
        })
    }
}

#[derive(Debug)]
// 個々人が持つwebhook
pub struct MemberWebhook {
    pub src_member_id: u64,
    pub dst_server_name: String,
    pub dst_guild_id: u64,
    pub dst_channel_id: u64,
    pub dst_webhook_url: String,
}

impl MemberWebhook {
    pub fn from(
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

    pub fn from_row(
        src_member_id: &str,
        dst_server_name: &str,
        dst_guild_id: &str,
        dst_channel_id: &str,
        dst_webhook_url: &str,
    ) -> Result<Self> {
        Ok(Self {
            src_member_id: src_member_id.parse()?,
            dst_server_name: dst_server_name.to_string(),
            dst_guild_id: dst_guild_id.parse()?,
            dst_channel_id: dst_channel_id.parse()?,
            dst_webhook_url: dst_webhook_url.to_string(),
        })
    }
}
