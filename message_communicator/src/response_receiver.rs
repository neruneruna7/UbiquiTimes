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
// use domain::traits::repositorys::OwnTimesRepository;

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

type ResReceiverResult<T> = Result<T, PoiseWebhookResReceiverError>;

#[derive(Debug, thiserror::Error)]
pub enum ResponseReceiverError {
    #[error("The server that received the response is not in the transmission record")]
    ServerNotInRecord,
}

enum ResMessageState {
    NotResMessage,
    ResMessage(ResponseMessage),
}

#[derive(Debug)]
pub struct PoiseWebhookResReceiver {
    other_times_repository: Arc<SledOtherTimesRepository>,
}

impl PoiseWebhookResReceiver {
    pub fn new(other_times_repository: Arc<SledOtherTimesRepository>) -> Self {
        Self {
            other_times_repository,
        }
    }

    fn is_bot_message(new_message: &poise::serenity_prelude::model::prelude::Message) -> bool {
        new_message.author.bot
    }

    fn deserialize_message(content: &str) -> ResMessageState {
        // デシリアライズし，メッセージがリクエストメッセージかどうかを判定
        match serde_json::from_str(content) {
            Ok(res) => ResMessageState::ResMessage(res),
            Err(_) => ResMessageState::NotResMessage,
        }
    }

    fn is_response_message(
        new_message: &poise::serenity_prelude::model::prelude::Message,
    ) -> ResMessageState {
        if !Self::is_bot_message(new_message) {
            return ResMessageState::NotResMessage;
        }
        Self::deserialize_message(&new_message.content)
    }

    fn is_sent_guild(
        sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
        res: &ResponseMessage,
    ) -> ResReceiverResult<String> {
        let is_response_from_sent_guild =
            Self::is_response_from_sent_guild(sent_member_and_guild_ids, res)?;

        // guild_nameを取得
        let guild_name = match is_response_from_sent_guild {
            Some(guild_name) => guild_name,
            None => {
                return Err(ResponseReceiverError::ServerNotInRecord.into());
            }
        };

        Ok(guild_name)
    }
}

impl UtResReceiver for PoiseWebhookResReceiver {
    type Error = PoiseWebhookResReceiverError;
    type NewMessage = poise::serenity_prelude::Message;

    async fn times_setting_response_receive(
        &self,
        new_message: &Self::NewMessage,
        sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
    ) -> Result<(), PoiseWebhookResReceiverError> {
        // botから以外のメッセージは無視
        if !new_message.author.bot {
            return Ok(());
        }

        // メッセージがリクエストメッセージかどうかを判定
        let res = Self::is_response_message(new_message);
        let res = match res {
            ResMessageState::NotResMessage => {
                return Ok(());
            }
            ResMessageState::ResMessage(res) => res,
        };

        // 送信記録にあるサーバからのレスポンスかどうかを判定する
        // ない場合はエラーを返す
        // guild_nameを取得
        let guild_name = Self::is_sent_guild(sent_member_and_guild_ids, &res)?;

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
