// Discordでのサーバーの呼び方に合わせて，OwnServerをOwnGuildに変更
// 同様の変更を他のにも
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct OwnGuild {
    pub guild_id: u64,
    pub guild_name: String,
    pub manage_channel_id: u64,
    pub manage_webhook_url: String,
    pub private_key_pem: String,
    pub public_key_pem: String,
}

impl OwnGuild {
    pub fn new(
        guild_id: u64,
        guild_name: &str,
        manage_channel_id: u64,
        manage_webhook_url: &str,
        private_key_pem: &str,
        public_key_pem: &str,
    ) -> Self {
        Self {
            guild_id,
            guild_name: guild_name.to_string(),
            manage_channel_id,
            manage_webhook_url: manage_webhook_url.to_string(),
            private_key_pem: private_key_pem.to_string(),
            public_key_pem: public_key_pem.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct OwnTimes {
    pub member_id: u64,
    pub member_name: String,
    pub channel_id: u64,
    pub times_webhook_url: String,
}

impl OwnTimes {
    pub fn new(
        member_id: u64,
        member_name: &str,
        channel_id: u64,
        times_webhook_url: &str,
    ) -> Self {
        Self {
            member_id,
            member_name: member_name.to_string(),
            channel_id,
            times_webhook_url: times_webhook_url.to_string(),
        }
    }
}

// 相手サーバーのデータ
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct OtherGuild {
    pub guild_id: u64,
    pub guild_name: String,
    pub webhook_url: String,
    pub public_key_pem: String,
}

impl OtherGuild {
    pub fn new(guild_id: u64, guild_name: &str, webhook_url: &str, public_key_pem: &str) -> Self {
        Self {
            guild_id,
            guild_name: guild_name.to_string(),
            webhook_url: webhook_url.to_string(),
            public_key_pem: public_key_pem.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
/// 他サーバのtimesデータ
// 主にTimesに書き込んだ内容の送信処理で使う
pub struct OtherTimes {
    pub src_member_id: u64,
    pub dst_guild_name: String,
    pub dst_guild_id: u64,
    pub dst_channel_id: u64,
    pub dst_webhook_url: String,
}

impl OtherTimes {
    pub fn new(
        src_member_id: u64,
        dst_guild_name: &str,
        dst_guild_id: u64,
        dst_channel_id: u64,
        dst_webhook_url: &str,
    ) -> Self {
        Self {
            src_member_id,
            dst_guild_name: dst_guild_name.to_string(),
            dst_guild_id,
            dst_channel_id,
            dst_webhook_url: dst_webhook_url.to_string(),
        }
    }
}
