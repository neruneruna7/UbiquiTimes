use serde::{Deserialize, Serialize};

use crate::own_server::OwnTimes;

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
    pub times_setting_response: TimesSettingResponse,
}

impl ResponseMessage {
    pub fn new(
        src_guild_id: u64,
        dst_guild_id: u64,
        times_setting_response: TimesSettingResponse,
    ) -> Self {
        Self {
            src_guild_id,
            dst_guild_id,
            times_setting_response,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TimesSettingRequest {
    pub req_src_member_id: u64,
    pub req_src_manage_webhook_url: String,
    pub req_src_channel_id: u64,
    pub req_src_member_webhook_url: String,
}

impl TimesSettingRequest {
    pub fn new(
        req_src_member_id: u64,
        req_src_manage_webhook_url: String,
        req_src_channel_id: u64,
        req_src_member_webhook_url: String,
    ) -> Self {
        Self {
            req_src_member_id,
            req_src_manage_webhook_url,
            req_src_channel_id,
            req_src_member_webhook_url,
        }
    }
}

// 常にリクエストの送信側をsrcとする
// AサーバがBサーバにリクエストを送信するとき，この構想体においてもAサーバがsrc，Bサーバがdstである
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimesSettingResponse {
    pub req_src_member_id: u64,
    pub req_dst_guild_id: u64,
    pub req_dst_member_channel_id: u64,
    pub req_dst_member_webhook_url: String,
}

impl TimesSettingResponse {
    pub fn from_req(req: &TimesSettingRequest, own_guild_id: u64, own_times: &OwnTimes) -> Self {
        Self {
            req_src_member_id: req.req_src_member_id,
            req_dst_guild_id: own_guild_id,
            req_dst_member_channel_id: own_times.channel_id,
            req_dst_member_webhook_url: own_times.times_webhook_url.clone(),
        }
    }
}
