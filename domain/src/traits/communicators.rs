use crate::models::{
    communication::{RequestMessage, ResponseMessage, TimesSettingRequest},
    guild_data::OwnGuild,
};

// できればpoiseへの依存がないトレイトを書きたい
pub trait UtReqSender {
    type Error;
    async fn times_setting_request_send(
        &self,
        own_server: &OwnGuild,
        dst_guild_id: u64,
        dst_guild_name: &str,
        req: TimesSettingRequest,
    ) -> Result<(), Self::Error>;
}

pub trait UtReqReceiver {
    type Error;

    async fn times_setting_receive_and_response(
        &self,
        // リクエストを受け取って，それに対するレスポンスを返すため
        // リクエストを引数にとる
        req: RequestMessage,
        own_guild_id: u64,
    ) -> Result<(), Self::Error>;
}

pub trait UtResReceiver {
    type Error;
    async fn times_setting_response_receive(&self, res: ResponseMessage)
        -> Result<(), Self::Error>;
}
