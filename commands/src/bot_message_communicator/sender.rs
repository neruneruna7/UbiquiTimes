use std::collections::HashSet;

use crate::bot_message;
use crate::bot_message::TimesSettingRequest;
use crate::global_data::Context;
use crate::other_server::OtherServer;
use crate::other_server_repository::OtherServerRepository;
use crate::sign;
use crate::sign::Claims;
use crate::sign::UbiquitimesSigner;

use super::TimesSettingCommunicatorResult;
use super::UbiquitimesSender;
use anyhow::Context as anyhowContext;
use poise::serenity_prelude::Http;
use poise::serenity_prelude::RwLock;
use poise::serenity_prelude::Webhook;

pub struct WebhookSender;

impl WebhookSender {
    pub fn new() -> Self {
        Self
    }

    // dbから他サーバのデータを取得
    async fn get_other_server(
        &self,
        ctx: &Context<'_>,
        dst_guild_id: u64,
    ) -> TimesSettingCommunicatorResult<crate::other_server::OtherServer> {
        let other_server = ctx
            .data()
            .other_server_repository
            .clone()
            .get_from_guild_id(dst_guild_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("OtherServer not found"))?;
        Ok(other_server)
    }

    // 送信につかうWebhookを取得
    async fn get_webhook(
        &self,
        ctx: &Context<'_>,
        other_server: &OtherServer,
    ) -> TimesSettingCommunicatorResult<Webhook> {
        // 送信だけなら特にトークン無しでもいいらしい
        // むしろトークン次第で何ができるのか気になるところ
        let http = Http::new("");
        let url = &other_server.webhook_url;
        let webhook = Webhook::from_url(http, url).await?;
        Ok(webhook)
    }

    // 送信するメッセージを作成
    async fn create_req_message(
        &self,
        ctx: &Context<'_>,
        dst_server: OtherServer,
        req: TimesSettingRequest,
    ) -> TimesSettingCommunicatorResult<bot_message::RequestMessage> {
        let own_guild_id = ctx.guild_id().unwrap().0;

        let _own_server = ctx.data().own_server_cache.read().await;
        let own_server = _own_server.as_ref().unwrap();

        let signer = sign::UbiquitimesPrivateKey::from_pem(&own_server.private_key_pem)
            .context("Failed to create private key")?;

        let claim = Claims::from_servers_for_req(own_server, &dst_server, req);

        let req_message = bot_message::RequestMessage::new(
            own_guild_id,
            dst_server.guild_id,
            signer.sign(claim).context("Failed to sign")?,
        );

        Ok(req_message)
    }

    // メッセージをシリアライズ
    fn serialize_req_message(
        &self,
        req_message: bot_message::RequestMessage,
    ) -> TimesSettingCommunicatorResult<String> {
        let req_message = serde_json::to_string(&req_message)?;
        Ok(req_message)
    }

    // 送信
    async fn send_webhook(
        &self,
        webhook: Webhook,
        req_message: String,
        ctx: &Context<'_>,
        dst_guild_id: u64,
    ) -> TimesSettingCommunicatorResult<()> {
        // どのサーバに対して送信したかを記録する
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

        // 送信
        webhook
            .execute(&ctx, false, |w| w.content(req_message))
            .await?;

        Ok(())
    }
}

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
        // dbから他サーバのデータを取得
        let other_server = self.get_other_server(ctx, dst_guild_id).await?;
        // 送信につかうWebhookを取得
        let webhook = self.get_webhook(ctx, &other_server).await?;

        // 送信するメッセージを作成
        let req_message = self.create_req_message(ctx, other_server, req).await?;

        // メッセージをシリアライズ
        let req_message = self.serialize_req_message(req_message)?;

        // 送信し，どのサーバに送信したかを記録する
        self.send_webhook(webhook, req_message, ctx, dst_guild_id)
            .await?;

        Ok(())
    }
}
