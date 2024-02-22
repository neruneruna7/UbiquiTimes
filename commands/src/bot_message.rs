use serde::{Deserialize, Serialize};

use crate::own_server::OwnServer;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestMessage {
    pub src_guild_id: u64,
    pub dst_guild_id: u64,
    /// Claimsに署名したもの
    /// base64エンコードされているので，その文字列を格納する
    /// signモジュールのClaimsを参照されたし
    pub jws_times_setting_request: String,
}

impl RequestMessage {
    pub fn new(src_guild_id: u64, dst_guild_id: u64, jws_times_setting_request: String) -> Self {
        Self {
            src_guild_id,
            dst_guild_id,
            jws_times_setting_request,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseMessage {
    pub src_guild_id: u64,
    pub dst_guild_id: u64,
    times_setting_responce: TimesSettingResponce,
}

impl ResponseMessage {
    pub fn new(
        src_guild_id: u64,
        dst_guild_id: u64,
        times_setting_responce: TimesSettingResponce,
    ) -> Self {
        Self {
            src_guild_id,
            dst_guild_id,
            times_setting_responce,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimesSettingRequest {
    pub req_src_member_id: u64,
    pub req_src_master_webhook_url: String,
    pub req_src_channel_id: u64,
    pub req_src_member_webhook_url: String,
}

// 常にリクエストの送信側をsrcとする
// AサーバがBサーバにリクエストを送信するとき，この構想体においてもAサーバがsrc，Bサーバがdstである
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimesSettingResponce {
    pub req_src_member_id: u64,
    pub req_dst_guild_id: u64,
    pub req_dst_channel_id: u64,
    pub req_dst_webhook_url: String,
}

impl TimesSettingResponce {
    pub fn from_req(req: &TimesSettingRequest, own_server: &OwnServer) -> Self {
        Self {
            req_src_member_id: req.src_member_id,
            req_dst_guild_id: own_server.guild_id,
            req_dst_channel_id: own_server.manage_channel_id,
            req_dst_webhook_url: own_server.manage_webhook_url.clone(),
        }
    }
}
