use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestMessage {
    pub own_guild_id: u64,
    pub other_guild_id: u64,
    /// Claimsに署名したもの
    /// base64エンコードされているので，その文字列を格納する
    /// signモジュールのClaimsを参照されたし
    pub jws_times_setting_request: String,
}

impl RequestMessage {
    pub fn new(own_guild_id: u64, other_guild_id: u64, jws_times_setting_request: String) -> Self {
        Self {
            own_guild_id,
            other_guild_id,
            jws_times_setting_request,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseMessage {
    pub own_guild_id: u64,
    pub other_guild_id: u64,
    times_setting_responce: TimesSettingResponce,
}

impl ResponseMessage {
    pub fn new(
        own_guild_id: u64,
        other_guild_id: u64,
        times_setting_responce: TimesSettingResponce,
    ) -> Self {
        Self {
            own_guild_id,
            other_guild_id,
            times_setting_responce,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimesSettingRequest {
    pub src_member_id: u64,
    pub src_master_webhook_url: String,
    pub src_channel_id: u64,
    pub src_member_webhook_url: String,
}

// 常にリクエストの送信側をsrcとする
// AサーバがBサーバにリクエストを送信するとき，この構想体においてもAサーバがsrc，Bサーバがdstである
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimesSettingResponce {
    pub src_member_id: u64,
    pub dst_guild_id: u64,
    pub dst_channel_id: u64,
    pub dst_webhook_url: String,
}
