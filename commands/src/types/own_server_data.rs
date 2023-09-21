use anyhow::{Error, Result};

pub struct ServerData {
    pub guild_id: u64,
    pub server_name: String,
    pub master_channel_id: u64,
    pub master_webhook_url: String,
}

impl ServerData {
    pub fn from(
        guild_id: u64,
        server_name: &str,
        master_channel_id: u64,
        master_webhook_url: &str,
    ) -> Self {
        Self {
            guild_id,
            server_name: server_name.to_string(),
            master_channel_id,
            master_webhook_url: master_webhook_url.to_string(),
        }
    }

    pub fn from_row(
        guild_id: &str,
        server_name: &str,
        master_channel_id: &str,
        master_webhook_url: &str,
    ) -> anyhow::Result<Self> {
        let guild_id = guild_id.parse::<u64>()?;
        let master_channel_id = master_channel_id.parse::<u64>()?;

        Ok(Self {
            guild_id,
            server_name: server_name.to_string(),
            master_channel_id,
            master_webhook_url: master_webhook_url.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct TimesData {
    pub member_id: u64,
    pub member_name: String,
    pub channel_id: u64,
    pub webhook_url: String,
}

impl TimesData {
    pub fn from(member_id: u64, member_name: &str, channel_id: u64, webhook_url: &str) -> Self {
        Self {
            member_id,
            member_name: member_name.to_string(),
            channel_id,
            webhook_url: webhook_url.to_string(),
        }
    }

    pub fn from_row(
        member_id: &str,
        member_name: &str,
        channel_id: &str,
        webhook_url: &str,
    ) -> Result<Self> {
        Ok(Self {
            member_id: member_id.parse::<u64>()?,
            member_name: member_name.to_string(),
            channel_id: channel_id.parse::<u64>()?,
            webhook_url: webhook_url.to_string(),
        })
    }
}
