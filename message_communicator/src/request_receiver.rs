use std::fmt::Debug;
use std::sync::Arc;

use ca_driver::my_ca_driver::MyCaDriverError;
use domain::models::communication::RequestMessage;
use domain::models::communication::ResponseMessage;
use domain::models::communication::TimesSettingResponse;

use ca_driver::my_ca_driver::MyCaDriver;
use domain::models::guild_data::OwnTimes;
use domain::models::sign::Claims;
use domain::models::sign::KeyAndWebhook;
use domain::thiserror;
use domain::thiserror::Error;
use domain::tracing;
use domain::tracing::info;
use domain::traits::repositorys::OwnTimesRepository;
use domain::traits::{
    ca_driver::CaDriver, communicators::UtReqReceiver, signer_verifier::UtVerifier,
};
use poise::serenity_prelude::ExecuteWebhook;
use poise::serenity_prelude::Http;
use poise::serenity_prelude::Message;
use poise::serenity_prelude::Webhook;
use signer_verifier::verifier::RsaVerifier;
use sled_repository::own_times_repository::SledOwnTimesRepository;

use crate::get_webhook::get_webhook;

#[derive(Debug, thiserror::Error)]
pub enum PoiseWebhookReqReceiverError {
    #[error("SelenityError: {0}")]
    SelenityError(#[from] poise::serenity_prelude::Error),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Verifier Error: {0}")]
    VerifierError(#[from] signer_verifier::verifier::VerifyError),
    #[error("Ca Driver Error: {0}")]
    CaDriverError(#[from] MyCaDriverError),
    #[error("Own Times Repository Error: {0}")]
    OwnTimesRepositoryError(
        #[from] sled_repository::own_times_repository::SledOwnTimesRepositoryError,
    ),
    #[error("Own Times Not Found: {0}")]
    OwnTimesNotFound(#[from] OwnTimesNotFound),
}

enum BotMessageState {
    NoBot,
    BotMessage(Message),
}

enum RequestMessageState {
    NotRequestMessage,
    RequestMessage(RequestMessage),
}
#[derive(Debug, Error)]
pub struct OwnTimesNotFound;

impl std::fmt::Display for OwnTimesNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OwnTimes not found")
    }
}

pub type ReqReceiverResult<T> = Result<T, PoiseWebhookReqReceiverError>;

/// 他サーバからのリクエストを受信する
///
#[derive(Debug)]
pub struct PoiseWebhookReqReceiver
// 試しにジェネリクスでコネコネしてみたけど，つらい
// where
//     C: CaDriver,
//     R: OwnTimesRepository,
{
    ca_driver: Arc<MyCaDriver>,
    own_times_repository: Arc<SledOwnTimesRepository>,
}

impl PoiseWebhookReqReceiver {
    pub fn new(
        ca_driver: Arc<MyCaDriver>,
        own_times_repository: Arc<SledOwnTimesRepository>,
    ) -> Self {
        Self {
            ca_driver,
            own_times_repository,
        }
    }

    // レスポンスを送信
    async fn send_res_message(
        &self,
        webhook: Webhook,
        serialized_message: String,
    ) -> ReqReceiverResult<()> {
        let http = Http::new("");
        let builder = ExecuteWebhook::new().content(serialized_message);
        webhook.execute(http, false, builder).await?;
        Ok(())
    }

    // Self::ってやるとサジェストが効いて楽だからimplimentのところに書く
    // どうせここ以外で使わない非公開関数だしね

    /// メッセージの送信者がbotであるかどうかを確認
    fn is_bot_message(new_message: poise::serenity_prelude::Message) -> BotMessageState {
        if !new_message.author.bot {
            return BotMessageState::NoBot;
        }
        BotMessageState::BotMessage(new_message)
    }

    /// new_messageをRequestMessageにデシリアライズ
    fn deserialize_message(new_message: poise::serenity_prelude::Message) -> RequestMessageState {
        // デシリアライズ
        let req: Result<RequestMessage, serde_json::Error> =
            serde_json::from_str(&new_message.content);

        match req {
            Ok(req) => {
                info!("ok:  new message is receive request");
                return RequestMessageState::RequestMessage(req);
            }
            Err(_e) => {
                info!("no:  new message is not receive request");
                return RequestMessageState::NotRequestMessage;
            }
        }
    }

    /// newmessageがRequestMessageかどうか調べる
    fn is_newmessage(new_message: poise::serenity_prelude::Message) -> RequestMessageState {
        let message = Self::is_bot_message(new_message);
        let message = match message {
            BotMessageState::NoBot => return RequestMessageState::NotRequestMessage,
            BotMessageState::BotMessage(message) => message,
        };
        let message = Self::deserialize_message(message);
        match message {
            RequestMessageState::NotRequestMessage => RequestMessageState::NotRequestMessage,
            RequestMessageState::RequestMessage(req_message) => {
                RequestMessageState::RequestMessage(req_message)
            }
        }
    }

    /// リクエストを検証する
    fn verify_request(
        req_message: &RequestMessage,
        key_and_webhook: &KeyAndWebhook,
    ) -> ReqReceiverResult<Claims> {
        let verifier = RsaVerifier::from_pem(&key_and_webhook.public_key_pem)?;
        let claim = verifier.verify(&req_message.jws_times_setting_request)?;
        info!("verify complete. claim: {:?}", claim);
        Ok(claim)
    }

    /// レスポンスメッセージを作る
    fn create_response(
        req_message: RequestMessage,
        claim: &Claims,
        own_guild_id: u64,
        own_times: &OwnTimes,
    ) -> ResponseMessage {
        let times_setting_response =
            TimesSettingResponse::from_req(&claim.times_setting_req, own_guild_id, &own_times);

        let response_message = ResponseMessage::new(
            req_message.src_guild_id,
            req_message.dst_guild_id,
            times_setting_response,
        );
        response_message
    }
}

impl UtReqReceiver for PoiseWebhookReqReceiver {
    type Result<T> = ReqReceiverResult<T>;
    type NewMessage = poise::serenity_prelude::Message;
    #[tracing::instrument(skip(self, new_message))]
    async fn times_setting_receive_and_response(
        &self,
        // リクエストを受け取って，それに対するレスポンスを返すため
        // リクエストを引数にとる
        new_message: &Self::NewMessage,
        own_guild_id: u64,
    ) -> Self::Result<()> {
        // Botからのリクエスト以外は無視する
        // Botからならデシリアライズする
        let req_message = Self::is_newmessage(new_message.clone());
        let req_message = match req_message {
            RequestMessageState::RequestMessage(req_message) => req_message,
            _ => return Ok(()),
        };

        // リクエストをを検証し，レスポンスを返す

        // // 送信元のサーバのwebhookと公開鍵を取得
        // // オレオレ認証局もどきにアクセスする
        let key_and_webhook = self
            .ca_driver
            .get_key_webhook(req_message.src_guild_id)
            .await?;

        // リクエストを検証
        let claim = Self::verify_request(&req_message, &key_and_webhook)?;

        // 必要なデータを取得
        let member_id = claim.times_setting_req.req_src_member_id;
        // use sled_repository::own_times_repository::SledOwnTimesRepositoryError::SledError.
        let own_times = self
            .own_times_repository
            .get(member_id)
            .await?
            .ok_or(OwnTimesNotFound)?;

        // レスポンスの作成
        let response_message = Self::create_response(req_message, &claim, own_guild_id, &own_times);

        // シリアライズ
        let serialized_message = serde_json::to_string(&response_message)?;

        // webhookを取得
        let webhook = get_webhook(&key_and_webhook.manage_webhook).await?;

        info!("send response message start");
        // レスポンスを送信
        self.send_res_message(webhook, serialized_message).await?;
        info!("send response message complete");

        Ok(())
    }
}
