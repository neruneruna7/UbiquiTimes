use crate::{
    bot_message::{RequestMessage, ResponseMessage, TimesSettingRequest},
    ca_driver::CaDriverError,
    global_data::{Context, Data},
};
use anyhow::Error;

use poise::serenity_prelude::{self as serenity, Message};
use tokio::sync::RwLock;
use tracing::{info, info_span};

use std::{collections::HashMap, fmt::Debug};
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
    // OwnServerRepositoryErrorを追加
    #[error("OwnServerRepository error: {0}")]
    OwnServerRepositoryError(#[from] crate::own_server_repository::OwnServerRepositoryError),
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

/// ハッシュマップに送信記録を保存するとき，キーとして使うための構造体
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct HashKey {
    member_id: u64,
    guild_id: u64,
}

impl HashKey {
    fn new(member_id: u64, guild_id: u64) -> Self {
        Self {
            member_id,
            guild_id,
        }
    }
}

/// どのサーバに対して送信したかを記録する
/// リクエストコマンド時に入力した識別用サーバー名も記録する必要が出てきた
async fn save_sent_guild_ids(
    ctx: &Context<'_>,
    dst_guild_id: u64,
    dst_guild_name: String,
) -> TimesSettingCommunicatorResult<()> {
    let mut sent_member_and_guild_ids = ctx.data().sent_member_and_guild_ids.write().await;

    let member_id = ctx.author().id.get();

    let hash_key = HashKey::new(member_id, dst_guild_id);

    info!(
        "hash_key: {:?}, server_name: {:?}",
        hash_key, dst_guild_name
    );

    // 一定時間後に削除するようにしたい

    sent_member_and_guild_ids.insert(hash_key, dst_guild_name);

    Ok(())
}

/// サーバからのレスポンスに対してリクエスト送信記録があるかどうか
/// 返ってくるStringはサーバ名
async fn is_response_from_sent_guild(
    framwework: poise::FrameworkContext<'_, crate::global_data::Data, anyhow::Error>,
    res: &ResponseMessage,
) -> TimesSettingCommunicatorResult<Option<String>> {
    let member_id = res.times_setting_response.req_src_member_id;
    let guild_id = res.dst_guild_id;

    let hash_key = HashKey::new(member_id, guild_id);

    // 該当データを取得
    let mut sent_member_and_guild_ids =
        framwework.user_data.sent_member_and_guild_ids.write().await;
    let sent_guild_name = sent_member_and_guild_ids.remove(&hash_key);

    info!(
        "hash_key: {:?}, server_name: {:?}",
        hash_key, sent_guild_name
    );

    Ok(sent_guild_name)
}

/// リクエストとレスポンス，両方のレシーバーを持つ構造体
///
/// 合成を試しに使ってみている
/// 合成ってのはこういうことでいいんだろうか
// 設計として良い悪いを判断できる知識がないので，とりあえず使ってみる
#[derive(Debug)]
pub struct MultiReceiver<T, R>
where
    T: UbiquitimesReqReceiver + Debug,
    R: UbiquitimesResReceiver + Debug,
{
    req_receiver: T,
    res_receiver: R,
}

impl<T, R> MultiReceiver<T, R>
where
    T: UbiquitimesReqReceiver + Debug,
    R: UbiquitimesResReceiver + Debug,
{
    pub fn new(req_receiver: T, res_receiver: R) -> Self {
        Self {
            req_receiver,
            res_receiver,
        }
    }

    #[tracing::instrument(skip(ctx, framework, new_message))]
    pub async fn receiv(
        &self,
        new_message: &Message,
        ctx: &serenity::Context,
        framework: poise::FrameworkContext<'_, Data, Error>,
    ) -> TimesSettingCommunicatorResult<()> {
        let msg_string = &new_message.content;

        // RequestMessageまたはResponseMessageに変換
        let result_req_message = serde_json::from_str::<RequestMessage>(msg_string);
        let result_res_message = serde_json::from_str::<ResponseMessage>(msg_string);
        info!("result_req_message: {:?}", result_req_message);
        info!("result_res_message: {:?}", result_res_message);

        // RequestMessageだった場合
        if let Ok(req_message) = result_req_message {
            info!("RequestMessage received");
            self.req_receiver
                .times_setting_receive_and_response(ctx, framework, req_message)
                .await?;

            // 早期リターンする
            return Ok(());
        }

        // ResponseMessageだった場合
        if let Ok(res_message) = result_res_message {
            info!("ResponseMessage received");
            self.res_receiver
                .times_setting_response_receive(framework, res_message)
                .await?;

            return Ok(());
        }

        // どちらでもなければ，その旨をログに残す
        info!("Not a bot message");

        Ok(())
    }
}
