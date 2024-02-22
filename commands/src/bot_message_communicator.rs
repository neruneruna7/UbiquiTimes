use crate::{
    bot_message::{RequestMessage, TimesSettingRequest, TimesSettingResponce},
    global_data::{Context, Data},
};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod reciever;
pub mod sender;
pub mod set;

#[derive(Debug, Error)]
pub enum TimesSettingCommunicatorError {
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    // 他のエラータイプもここに追加できます
    #[error("Serenity error: {0}")]
    SerenityError(#[from] serenity::Error),
    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("sign error: {0}")]
    SignError(#[from] crate::sign::SignError),
}

pub type TimesSettingCommunicatorResult<T> = Result<T, TimesSettingCommunicatorError>;

pub trait UbiquitimesSender {
    async fn times_setting_request_send(
        &self,
        ctx: &Context<'_>,
        dst_guild_id: u64,
        req: TimesSettingRequest,
    ) -> TimesSettingCommunicatorResult<()>;
}

pub trait UbiquitimesReciever {
    async fn times_setting_recieve_and_response(
        &self,
        // poiseのContextが使えないので，serenityのContextを使う
        ctx: &serenity::Context,
        _framework: poise::FrameworkContext<'_, Data, anyhow::Error>,
        // リクエストを受け取って，それに対するレスポンスを返すため
        // リクエストを引数にとる
        req: RequestMessage,
    ) -> TimesSettingCommunicatorResult<()>;
}
