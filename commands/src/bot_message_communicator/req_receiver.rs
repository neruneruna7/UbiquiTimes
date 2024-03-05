use crate::bot_message;
use crate::bot_message::ResponseMessage;
use crate::bot_message::TimesSettingResponse;
use crate::ca_driver::my_ca_driver::MyCaDriver;
use crate::ca_driver::CaDriver;
use crate::ca_driver::KeyAndWebhook;
use crate::global_data;

use crate::other_server_repository::OtherServerRepository;

use crate::own_server_repository::OwnServerRepository;
use crate::sign;
use crate::sign::Claims;

use crate::sign::UbiquitimesVerifier;

use super::TimesSettingCommunicatorResult;
use super::UbiquitimesReqReceiver;
use anyhow::Context as anyhowContext;
use poise::serenity_prelude::ExecuteWebhook;
use poise::serenity_prelude::Http;
use poise::serenity_prelude::Webhook;
use rsa::pkcs8::SubjectPublicKeyInfoRef;
use tracing::info;

/// 他サーバからのリクエストを受信する
///
#[derive(Debug)]
pub struct WebhookReqReceiver;

impl WebhookReqReceiver {
    pub fn check(new_message: &poise::serenity_prelude::Message) -> bool {
        // ここでリクエストのチェックを行う
        // botから以外のメッセージは無視する
        if !new_message.author.bot {
            return false;
        }
        true
    }

    // 認証局もどきからリクエスト送信元サーバのデータを取得
    async fn get_src_key_and_webhook(
        guild_id: u64,
    ) -> TimesSettingCommunicatorResult<KeyAndWebhook> {
        let ca_driver = MyCaDriver::new();
        let key_and_webhook = ca_driver.get_key_webhook(guild_id).await?;
        Ok(key_and_webhook)
    }

    // リクエストを検証して，Claimsを取得する
    async fn verify(
        &self,
        _framework: poise::FrameworkContext<'_, global_data::Data, anyhow::Error>,
        req: &bot_message::RequestMessage,
        public_key_pem: &str,
    ) -> TimesSettingCommunicatorResult<Claims> {
        info!("CA access complete. public_key_pem: {}", public_key_pem);

        let verifier = sign::UbiquitimesPublicKey::from_pem(&public_key_pem)
            .context("Failed to create verifier")?;

        info!("verifier created.");

        // リクエストを検証
        let claim = verifier
            .verify(&req.jws_times_setting_request)
            .context(format!(
                "Failed to Verifey, src_guild_id is {} ,検証に失敗しました",
                req.src_guild_id,
            ))?;

        info!("verify complete. claim: {:?}", claim);
        Ok(claim)
    }

    // 必要なデータを取得
    async fn get_own_server(
        &self,
        framework: poise::FrameworkContext<'_, global_data::Data, anyhow::Error>,
    ) -> TimesSettingCommunicatorResult<crate::own_server::OwnServer> {
        let own_server_repository = framework.user_data.own_server_repository.clone();
        let own_server = own_server_repository.get().await?;

        Ok(own_server.clone())
    }

    // レスポンスを作成し，シリアライズ
    async fn create_res_message(
        &self,
        req: &bot_message::RequestMessage,
        claim: &Claims,
        own_server: &crate::own_server::OwnServer,
    ) -> TimesSettingCommunicatorResult<String> {
        // レスポンスの作成
        let setting_res = TimesSettingResponse::from_req(&claim.times_setting_req, own_server);
        let res_message = ResponseMessage::new(req.src_guild_id, req.dst_guild_id, setting_res);

        let serialized_message = serde_json::to_string(&res_message)?;
        Ok(serialized_message)
    }

    // webhookを取得
    async fn create_webhook(&self, webhook_url: &str) -> TimesSettingCommunicatorResult<Webhook> {
        let webhook = {
            let http = Http::new("");
            Webhook::from_url(http, webhook_url).await?
        };

        Ok(webhook)
    }

    // レスポンスを送信
    async fn send_res_message(
        &self,
        ctx: &poise::serenity_prelude::Context,
        webhook: Webhook,
        serialized_message: String,
    ) -> TimesSettingCommunicatorResult<()> {
        let builder = ExecuteWebhook::new().content(serialized_message);
        webhook.execute(ctx, false, builder).await?;
        Ok(())
    }
}

impl UbiquitimesReqReceiver for WebhookReqReceiver {
    #[tracing::instrument(skip(self, ctx, framework, req))]
    async fn times_setting_receive_and_response(
        &self,
        // poiseのContextが使えないので，serenityのContextを使う
        ctx: &poise::serenity_prelude::Context,
        framework: poise::FrameworkContext<'_, global_data::Data, anyhow::Error>,
        // リクエストを受け取って，それに対するレスポンスを返すため
        // リクエストを引数にとる
        req: bot_message::RequestMessage,
    ) -> TimesSettingCommunicatorResult<()> {
        // リクエストをを検証し，レスポンスを返す
        info!("receive request start");

        // 送信元のサーバのwebhookと公開鍵を取得
        // オレオレ認証局もどきにアクセスする
        let key_and_webhook = Self::get_src_key_and_webhook(req.src_guild_id).await?;

        // リクエストを検証
        let claim = self
            .verify(framework, &req, &key_and_webhook.public_key_pem)
            .await
            .context("Failed to verify request")?;
        info!("verify complete. claim: {:?}", claim);

        // 必要なデータを取得
        let own_server = self.get_own_server(framework).await?;

        // レスポンスの作成とシリアライズ
        let serialized_message = self
            .create_res_message(&req, &claim, &own_server)
            .await
            .context("Failed to create response message")?;

        // webhookを取得
        let webhook = self.create_webhook(&key_and_webhook.manage_webhook).await?;

        info!("send response message start");
        // レスポンスを送信
        self.send_res_message(ctx, webhook, serialized_message)
            .await
            .context("Failed to send response message")?;
        info!("send response message complete");

        Ok(())
    }
}
