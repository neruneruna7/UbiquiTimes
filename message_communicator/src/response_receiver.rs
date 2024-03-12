use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

// レスポンスを受け取って，それを処理する

use domain::models::communication::ResponseMessage;

use domain::models::guild_data::OtherTimes;

use domain::thiserror;

use domain::traits::communicators::GuildName;
use domain::traits::communicators::HashKey;
use domain::traits::communicators::UtResReceiver;
use domain::traits::repositorys::OtherTimesRepository;
use domain::traits::repositorys::OwnTimesRepository;

use sled_repository::other_times_repository::SledOtherTimesRepository;

#[derive(Debug, thiserror::Error)]
pub enum PoiseWebhookResReceiverError {
    #[error("SelenityError: {0}")]
    SelenityError(#[from] poise::serenity_prelude::Error),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Verifier Error: {0}")]
    VerifierError(#[from] signer_verifier::verifier::VerifyError),
    #[error("Other Times Repository Error: {0}")]
    OtherTimesRepositoryError(
        #[from] sled_repository::other_times_repository::SledOtherTimesRepositoryError,
    ),
    #[error("Response Receiver Error: {0}")]
    ResponseReceiverError(#[from] ResponseReceiverError),
}

#[derive(Debug, thiserror::Error)]
pub enum ResponseReceiverError {
    #[error("The server that received the response is not in the transmission record")]
    ServerNotInRecord,
}

#[derive(Debug)]
pub struct PoiseWebhookResReceiver<R>
where
    R: OtherTimesRepository,
{
    other_times_repository: R,
}

impl<R> PoiseWebhookResReceiver<R>
where
    R: OtherTimesRepository,
{
    pub fn new(other_times_repository: R) -> Self {
        Self {
            other_times_repository,
        }
    }
}

impl UtResReceiver for PoiseWebhookResReceiver<SledOtherTimesRepository> {
    type Error = PoiseWebhookResReceiverError;

    type NewMessage = poise::serenity_prelude::Message;

    async fn times_setting_response_receive(
        &self,
        new_message: Self::NewMessage,
        sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
    ) -> Result<(), PoiseWebhookResReceiverError> {
        // botから以外のメッセージは無視
        if !new_message.author.bot {
            return Ok(());
        }

        // メッセージがリクエストメッセージかどうかを判定
        let res: ResponseMessage = match serde_json::from_str(&new_message.content) {
            Ok(res) => res,
            Err(_) => {
                return Ok(());
            }
        };

        // 送信記録にあるサーバからのレスポンスかどうかを判定する
        // ない場合はエラーを返す
        let is_response_from_sent_guild =
            Self::is_response_from_sent_guild(sent_member_and_guild_ids, &res)?;
        let guild_name = match is_response_from_sent_guild {
            Some(guild_name) => guild_name,
            None => {
                return Err(ResponseReceiverError::ServerNotInRecord.into());
            }
        };

        // レスポンスを処理
        // 相手のサーバを拡散先サーバとして登録
        // または，拡散先サーバの情報を更新

        // OtherTimes構造体を作成
        let other_times = OtherTimes::new(
            res.times_setting_response.req_src_member_id,
            &guild_name,
            res.times_setting_response.req_dst_guild_id,
            res.times_setting_response.req_dst_member_channel_id,
            &res.times_setting_response.req_dst_member_webhook_url,
        );

        // DBに登録

        self.other_times_repository.upsert(other_times).await?;

        Ok(())
    }
}
