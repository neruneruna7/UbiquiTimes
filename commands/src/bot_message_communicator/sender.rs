use std::collections::HashSet;

use crate::bot_message;
use crate::global_data::Context;
use crate::other_server_repository::OtherServerRepository;
use crate::sign;
use crate::sign::Claims;
use crate::sign::UbiquitimesSigner;

use super::TimesSettingCommunicatorResult;
use super::UbiquitimesSender;
use anyhow::Context as anyhowContext;
use poise::serenity_prelude::Http;
use poise::serenity_prelude::Webhook;

pub struct WebhookSender;

impl UbiquitimesSender for WebhookSender {
    /// 他サーバにリクエストを送信する
    ///
    ///
    async fn times_setting_request_send(
        &self,
        ctx: &Context<'_>,
        dst_guild_id: u64,
        req: crate::bot_message::TimesSettingRequest,
    ) -> TimesSettingCommunicatorResult<()> {
        // let mut sended_manage_webhook = HashSet::new();

        // let url =
        // let webhook = Webhook::from_url(http, url)

        // dbから他サーバのデータを取得
        let other_server = ctx
            .data()
            .other_server_repository
            .clone()
            .get_from_guild_id(dst_guild_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("OtherServer not found"))?;

        // 送信につかうWebhookを取得
        let webhook = {
            // 送信だけなら特にトークン無しでもいいらしい
            // むしろトークン次第で何ができるのか気になるところ
            let http = Http::new("");
            let url = &other_server.webhook_url;
            Webhook::from_url(http, url).await?
        };

        // 送信するメッセージを作成
        let req_message = {
            let own_guild_id = ctx.guild_id().unwrap().0;

            let _own_server = ctx.data().own_server_cache.read().await;
            let own_server = _own_server.as_ref().unwrap();

            let signer = sign::UbiquitimesPrivateKey::from_pem(&own_server.private_key_pem)
                .context("Failed to create private key")?;

            let claim = Claims::from_servers_for_req(own_server, &other_server, req);

            let req_message = bot_message::RequestMessage::new(
                own_guild_id,
                dst_guild_id,
                signer.sign(claim).context("Failed to sign")?,
            );

            req_message
        };

        // メッセージをシリアライズ
        let req_message = serde_json::to_string(&req_message)?;

        // 送信
        webhook
            .execute(&ctx, false, |w| w.content(req_message))
            .await?;

        Ok(())
    }
}
