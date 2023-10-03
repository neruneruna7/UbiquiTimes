
use serde::{Deserialize, Serialize};

pub mod recieved;
pub mod send;
pub mod set;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotComMessage {
    pub src_guild_id: u64,
    pub dst_guild_id: u64,
    /// Claimsに署名したもの
    pub token: Option<String>,
    pub cmd_kind: Option<CmdKind>,
}

impl BotComMessage {
    pub fn new(
        src_guild_id: u64,
        dst_guild_id: u64,
        token: Option<String>,
        cmd_kind: Option<CmdKind>,
    ) -> BotComMessage {
        Self {
            src_guild_id,
            dst_guild_id,
            token,
            cmd_kind,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotComMessage2 {
    pub src_guild_id: u64,
    pub dst_guild_id: u64,
    pub cmd_kind: CmdKind,
}

impl BotComMessage2 {
    pub fn new(src_guild_id: u64, dst_guild_id: u64, cmd_kind: CmdKind) -> BotComMessage2 {
        Self {
            src_guild_id,
            dst_guild_id,
            cmd_kind,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CmdKind {
    TimesUbiquiSettingSend(TimesUbiquiSettingSend),
    TimesUbiquiSettingRecv(TimesUbiquiSettingRecv),
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimesUbiquiSettingSend {
    pub src_member_id: u64,
    pub src_master_webhook_url: String,
    pub src_channel_id: u64,
    pub src_member_webhook_url: String,
}

// 常にリクエストの送信側をsrcとする
// AサーバがBサーバにリクエストを送信するとき，この構想体においてもAサーバがsrc，Bサーバがdstである
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimesUbiquiSettingRecv {
    pub src_member_id: u64,
    pub dst_guild_id: u64,
    pub dst_channel_id: u64,
    pub dst_webhook_url: String,
}
