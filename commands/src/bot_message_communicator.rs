use crate::{
    bot_message::{RequestMessage, ResponseMessage, TimesSettingRequest, TimesSettingResponse},
    ca_driver::CaDriverError,
    global_data::{Context, Data},
};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

pub mod req_receiver;
pub mod req_sender;
pub mod res_receiver;

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
    #[error("OtherServerRepository error: {0}")]
    OtherServerRepositoryError(#[from] crate::other_server_repository::OtherServerRepositoryError),
    #[error("CaDriver error: {0}")]
    CaDriverError(#[from] CaDriverError),
}

pub type TimesSettingCommunicatorResult<T> = Result<T, TimesSettingCommunicatorError>;

pub trait UbiquitimesReqSender {
    async fn times_setting_request_send(
        &self,
        ctx: &Context<'_>,
        dst_guild_id: u64,
        dst_guild_name: &str,
        req: TimesSettingRequest,
    ) -> TimesSettingCommunicatorResult<()>;
}

pub trait UbiquitimesReqReceiver {
    async fn times_setting_receive_and_response(
        &self,
        // poiseのContextが使えないので，serenityのContextを使う
        ctx: &serenity::Context,
        _framework: poise::FrameworkContext<'_, Data, anyhow::Error>,
        // リクエストを受け取って，それに対するレスポンスを返すため
        // リクエストを引数にとる
        req: RequestMessage,
    ) -> TimesSettingCommunicatorResult<()>;
}

pub trait UbiquitimesResReceiver {
    async fn times_setting_response_receive(
        &self,
        framwework: poise::FrameworkContext<'_, Data, anyhow::Error>,
        res: ResponseMessage,
    ) -> TimesSettingCommunicatorResult<()>;
}

// どのサーバに対して送信したかを記録する
async fn save_sent_guild_ids(
    ctx: &Context<'_>,
    dst_guild_id: u64,
) -> TimesSettingCommunicatorResult<()> {
    let mut sent_member_and_guild_ids = ctx.data().sent_member_and_guild_ids.write().await;

    let member_id = ctx.author().id.0;
    // メンバーごとに紐づく送信記録がまだなければ作成
    let sent_guild_ids = sent_member_and_guild_ids.get(&member_id);

    let sent_guild_ids = match sent_guild_ids {
        Some(sent_guild_ids) => sent_guild_ids,
        None => {
            let sent_guild_ids = RwLock::new(HashSet::new());
            sent_member_and_guild_ids.insert(member_id, sent_guild_ids);
            sent_member_and_guild_ids.get(&member_id).unwrap()
        }
    };
    // 送信記録を更新
    sent_guild_ids.write().await.insert(dst_guild_id);

    Ok(())
}

// 送信記録にあるサーバからのレスポンスかどうかを判定する
async fn is_response_from_sent_guild(
    framwework: poise::FrameworkContext<'_, crate::global_data::Data, anyhow::Error>,
    res: &ResponseMessage,
) -> TimesSettingCommunicatorResult<bool> {
    let member_id = res.times_setting_response.req_src_member_id;
    let guild_id = res.src_guild_id;

    // 該当データを取得
    let sent_member_and_guild_ids = framwework.user_data.sent_member_and_guild_ids.read().await;
    let sent_guild_ids = sent_member_and_guild_ids.get(&member_id);

    let is_response_from_sent_guild = match sent_guild_ids {
        Some(sent_guild_ids) => {
            // guild_idが一致するものがあれば，その記録を削除し，trueを返す
            let mut sent_guild_ids = sent_guild_ids.write().await;
            let is_response_from_sent_guild = sent_guild_ids.remove(&guild_id);
            is_response_from_sent_guild
        }
        None => false,
    };

    Ok(is_response_from_sent_guild)
}
