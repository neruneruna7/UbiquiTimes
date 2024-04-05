use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use ca_driver::my_ca_driver::MyCaDriver;
use ca_driver::my_ca_driver::MyCaDriverError;
use domain::models::communication::RequestMessage;

use domain::models::communication::TimesSettingRequest;
use domain::models::guild_data::OtherGuild;

use domain::models::guild_data::OwnGuild;

use domain::models::sign::Claims;
use domain::traits::ca_driver::CaDriver;
use domain::traits::communicators::GuildName;
use domain::traits::communicators::HashKey;
use domain::traits::communicators::UtReqSender;

use domain::traits::signer_verifier::UtSigner;
use poise::serenity_prelude::ExecuteWebhook;
use poise::serenity_prelude::Http;

use domain::thiserror;

use signer_verifier::signer::RsaSigner;

use crate::get_webhook::get_webhook;

#[derive(Debug, thiserror::Error)]
pub enum PoiseWebhookReqSenderError {
    #[error("SelenityError: {0}")]
    SelenityError(#[from] poise::serenity_prelude::Error),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Sign Error: {0}")]
    SignError(#[from] signer_verifier::signer::SignError),
    #[error("Ca Driver Error: {0}")]
    CaDriverError(#[from] MyCaDriverError),
}

pub type ReqSenderResult<T> = Result<T, PoiseWebhookReqSenderError>;

#[derive(Debug)]
pub struct PoiseWebhookReqSender {
    ca_driver: Arc<MyCaDriver>,
}

impl PoiseWebhookReqSender {
    pub fn new(ca_driver: Arc<MyCaDriver>) -> Self {
        Self { ca_driver }
    }

    // メッセージをシリアライズ
    fn serialize_req_message(req_message: RequestMessage) -> ReqSenderResult<String> {
        // そもそも単一の機能 ラッパーのような関数を作る意味はあるだろうか？
        let req_message = serde_json::to_string(&req_message)?;
        Ok(req_message)
    }

    // 送信するメッセージを作成
    fn sign_craim(own_guild: &OwnGuild, claim: Claims) -> ReqSenderResult<String> {
        // 特定の手順であり，手順間で値の受け渡しをする
        // 逐次的凝集とみなせるだろう
        let signer = RsaSigner::from_pem(&own_guild.private_key_pem)?;
        let signed_claim = signer.sign(claim)?;
        Ok(signed_claim)
    }

    fn create_req_message(
        own_guild: &OwnGuild,
        dst_guild: &OtherGuild,
        req: TimesSettingRequest,
    ) -> ReqSenderResult<RequestMessage> {
        // 手順間で同じ値を使う部分がある？（own_guild）
        // 通信的凝集...?
        // 違う気もする
        // わからない
        let claim = Claims::from_servers_for_req(own_guild, dst_guild, req);
        let signed_claim = Self::sign_craim(own_guild, claim)?;
        let req_message = RequestMessage::new(own_guild.guild_id, dst_guild.guild_id, signed_claim);
        Ok(req_message)
    }

    /// 指定したギルドに送信する
    /// どこに(dst_guild)，何を(req_message)
    async fn send_message(dst_guild: &OtherGuild, req_message: String) -> ReqSenderResult<()> {
        // 逐次的凝集...か？
        // 送信につかうWebhookを作成
        let webhook = get_webhook(&dst_guild.webhook_url).await?;
        let http = Http::new("");
        // 送信
        let builder = ExecuteWebhook::new().content(req_message);
        webhook.execute(&http, false, builder).await?;
        Ok(())
    }

    fn save_sent_guild(
        sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
        member_id: u64,
        dst_guild: OtherGuild,
    ) {
        // 凝集がわからなくなってきた
        // 逐次的凝集だろうか？
        let dst_guild_id = dst_guild.guild_id;
        let dst_guild_name = dst_guild.guild_name.to_string();
        Self::save_sent_guild_ids(
            sent_member_and_guild_ids,
            member_id,
            dst_guild_id,
            dst_guild_name,
        );
    }
}

impl UtReqSender for PoiseWebhookReqSender {
    type Result<T> = ReqSenderResult<T>;
    /// 他サーバにリクエストを送信する
    ///
    /// dst_guild_idは送信先のサーバのID かならず機械的にどのサーバか特定できるもの
    /// dst_guild_nameは送信先のサーバの名前 人間が識別可能であればなんでもよい
    async fn times_setting_request_send(
        //
        &self,
        own_guild: &OwnGuild,
        dst_guild_id: u64,
        dst_guild_name: &str,
        member_id: u64,
        times_setting_req: TimesSettingRequest,
        sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
    ) -> Self::Result<()> {
        // 認証局もどきからリクエスト送信先の公開鍵とmanage_webhookを取得
        let key_and_webhook = self.ca_driver.get_key_webhook(dst_guild_id).await?;

        let dst_guild = OtherGuild::new(
            dst_guild_id,
            dst_guild_name,
            &key_and_webhook.manage_webhook,
            &key_and_webhook.public_key_pem,
        );

        // リクエストをメッセージを作成
        let req_message = Self::create_req_message(own_guild, &dst_guild, times_setting_req)?;

        // メッセージをシリアライズ
        let req_message = Self::serialize_req_message(req_message)?;

        // 送信する
        Self::send_message(&dst_guild, req_message).await?;

        // どのサーバに送信したかを記録する
        Self::save_sent_guild(sent_member_and_guild_ids, member_id, dst_guild);

        Ok(())
    }
}
