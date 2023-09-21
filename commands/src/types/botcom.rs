use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BotComMessage {
    pub src: String,
    pub dst: String,
    pub cmd: CmdKind,
    pub ttl: usize,
}

impl BotComMessage {
    pub fn from(src: &str, dst: &str, cmd: CmdKind) -> BotComMessage {
        let src = src.to_string();
        let dst = dst.to_string();
        let ttl = 4;
        Self { src, dst, cmd, ttl }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CmdKind {
    TimesUbiquiSettingSend(TimesUbiquiSettingSend),
    TimesUbiquiSettingRecv(TimesUbiquiSettingRecv),
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimesUbiquiSettingSend {
    pub src_member_id: u64,
    pub src_master_webhook_url: String,
    pub src_channel_id: u64,
    pub src_member_webhook_url: String,
}

// 常にリクエストの送信側をsrcとする
// AサーバがBサーバにリクエストを送信するとき，この構想体においてもAサーバがsrc，Bサーバがdstである
#[derive(Debug, Serialize, Deserialize)]
pub struct TimesUbiquiSettingRecv {
    pub src_member_id: u64,
    pub dst_guild_id: u64,
    pub dst_channel_id: u64,
    pub dst_webhook_url: String,
}
