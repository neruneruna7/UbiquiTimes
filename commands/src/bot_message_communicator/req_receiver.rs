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
    ) -> TimesSettingCommunicatorResult<Claims> {
        // 送信元のサーバの公開鍵を取得
        // オレオレ認証局もどきにアクセスする
        let public_key_pem = {
            let key_and_webhook = Self::get_src_key_and_webhook(req.src_guild_id).await?;
            key_and_webhook.public_key_pem
        };

        let verifier = sign::UbiquitimesPublicKey::from_pem(&public_key_pem)
            .context("Failed to create verifier")?;

        // リクエストを検証
        let claim = verifier
            .verify(&req.jws_times_setting_request)
            .context(format!(
                "Failed to Verifey, src_guild_id is {} ,検証に失敗しました",
                req.src_guild_id,
            ))?;
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
    async fn get_webhook(
        &self,
        framework: poise::FrameworkContext<'_, global_data::Data, anyhow::Error>,
        req: &bot_message::RequestMessage,
    ) -> TimesSettingCommunicatorResult<Webhook> {
        let webhook = {
            let http = Http::new("");
            // リクエストを送信したサーバのWebhookを取得
            let webhook_url = {
                let other_server = framework
                    .user_data
                    .other_server_repository
                    .clone()
                    .get_from_guild_id(req.src_guild_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("OtherServer guild_id: {} is not found", req.src_guild_id)
                    })?;
                other_server.webhook_url
            };

            Webhook::from_url(http, &webhook_url).await?
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

        // リクエストを検証
        let claim = self
            .verify(framework, &req)
            .await
            .context("Failed to verify request")?;

        // 必要なデータを取得
        let own_server = self.get_own_server(framework).await?;

        // レスポンスの作成とシリアライズ
        let serialized_message = self
            .create_res_message(&req, &claim, &own_server)
            .await
            .context("Failed to create response message")?;

        // webhookを取得
        let webhook = self.get_webhook(framework, &req).await?;

        // レスポンスを送信
        self.send_res_message(ctx, webhook, serialized_message)
            .await
            .context("Failed to send response message")?;

        Ok(())
    }
}
