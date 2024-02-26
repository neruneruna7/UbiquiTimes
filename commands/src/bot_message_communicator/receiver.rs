use crate::bot_message;
use crate::bot_message::ResponseMessage;
use crate::bot_message::TimesSettingResponse;
use crate::global_data::Context;
use crate::other_server_repository::OtherServerRepository;
use crate::sign;
use crate::sign::Claims;
use crate::sign::UbiquitimesSigner;
use crate::sign::UbiquitimesVerifier;

use super::TimesSettingCommunicatorResult;
use super::UbiquitimesReceiver;
use anyhow::Context as anyhowContext;
use poise::serenity_prelude::Http;
use poise::serenity_prelude::Webhook;
use serde::Serialize;

/// 他サーバからのリクエストを受信する
///
pub struct WebhookReceiver;

impl WebhookReceiver {
    pub fn check(new_message: &poise::serenity_prelude::Message) -> bool {
        // ここでリクエストのチェックを行う
        // botから以外のメッセージは無視する
        if !new_message.author.bot {
            return false;
        }
        true
    }
    
}

impl UbiquitimesReceiver for WebhookReceiver {
    async fn times_setting_receive_and_response(
        &self,
        // poiseのContextが使えないので，serenityのContextを使う
        ctx: &poise::serenity_prelude::Context,
        framework: poise::FrameworkContext<'_, crate::global_data::Data, anyhow::Error>,
        // リクエストを受け取って，それに対するレスポンスを返すため
        // リクエストを引数にとる
        req: bot_message::RequestMessage,
    ) -> TimesSettingCommunicatorResult<()> {
        // リクエストをを検証し，レスポンスを返す

        // リクエストを検証
        let public_key_pem = {
            let src_guild_id = req.src_guild_id;
            let other_server = framework
                .user_data
                .other_server_repository
                .clone()
                .get_from_guild_id(src_guild_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("SrcServer [id]:{} is not found", src_guild_id))?;
            other_server.public_key_pem
        };
        let verifier = sign::UbiquitimesPublicKey::from_pem(&public_key_pem)
            .context("Failed to create verifier")?;

        let claim = verifier
            .verify(&req.jws_times_setting_request)
            .context(format!(
                "Failed to Verifey, src_guild_id is {} ,検証に失敗しました",
                req.src_guild_id,
            ))?;
        // 必要なデータを取得

        let _own_server = framework.user_data.own_server_cache.read().await;
        let own_server = _own_server
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Own Server Data is Not Found"))?;

        // レスポンスの作成
        let setting_res = TimesSettingResponse::from_req(&claim.times_setting_req, own_server);
        let res_message = ResponseMessage::new(req.src_guild_id, req.dst_guild_id, setting_res);

        // シリアライズ
        let serialized_message = serde_json::to_string(&res_message)?;

        // レスポンスを送信
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

        webhook
            .execute(ctx, false, |w| w.content(serialized_message))
            .await?;

        Ok(())

    }
}
