use serde::{Deserialize, Serialize};

use super::{
    communication::TimesSettingRequest,
    guild_data::{OtherGuild, OwnGuild},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
// なんだったか忘れたけど，何かに基づいてClaim型を定義した
// 署名のときの云々があったはず...
// jwsクレートのドキュメントに基づいて決めた
pub struct Claims {
    // 送信元サーバ名
    pub iss: String,
    // GUILD_ID
    // ...どっちのだっけ？
    pub sub: u64,
    // 送信先サーバ名
    pub aud: String,
    pub exp: usize,
    pub times_setting_req: TimesSettingRequest,
}

impl Claims {
    pub fn new(iss: &str, sub: u64, aud: &str, times_setting_req: TimesSettingRequest) -> Claims {
        let iss = iss.to_string();
        let aud = aud.to_string();
        let exp = 10000000000;
        Self {
            iss,
            sub,
            aud,
            exp,
            times_setting_req,
        }
    }

    /// リクエスト送信時につかうClaimsをサーバーデータから生成する
    pub fn from_servers_for_req(
        own_server: &OwnGuild,
        other_server: &OtherGuild,
        times_setting_req: TimesSettingRequest,
    ) -> Self {
        let iss = own_server.guild_name.clone();
        let sub = own_server.guild_id;
        let aud = other_server.guild_name.clone();
        let exp = 10000000000;
        Self {
            iss,
            sub,
            aud,
            exp,
            times_setting_req,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyAndWebhook {
    pub guild_id: u64,
    pub public_key_pem: String,
    pub manage_webhook: String,
}
