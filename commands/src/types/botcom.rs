use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BotComMessage {
    pub src: String,
    pub dst: String,
    pub cmd: CmdKind,
    pub ttl: usize,
}

pub struct Claims {
    // 送信元サーバ名
    pub iss: String,
    // GUILD_ID
    pub sub: String,
    // 送信先サーバ名
    pub aud: String,
    pub exp: usize,
    pub cmdind: CmdKind,
}

impl Claims {
    pub fn new(iss: &str, sub: &str, aud: &str, cmdind: CmdKind) -> Claims {
        let iss = iss.to_string();
        let sub = sub.to_string();
        let aud = aud.to_string();
        let exp = 10000000000;
        Self {
            iss,
            sub,
            aud,
            exp,
            cmdind,
        }
    }
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
