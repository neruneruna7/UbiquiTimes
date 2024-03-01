use crate::bot_message;
use crate::bot_message::TimesSettingRequest;
use crate::ca_driver::my_ca_driver::MyCaDriver;
use crate::ca_driver::CaDriver;
use crate::global_data::Context;
use crate::other_server::OtherServer;

use crate::sign;
use crate::sign::Claims;
use crate::sign::UbiquitimesSigner;

use super::save_sent_guild_ids;
use super::TimesSettingCommunicatorResult;
use super::UbiquitimesReqSender;
use anyhow::Context as anyhowContext;
use poise::serenity_prelude::Http;

use poise::serenity_prelude::Webhook;

pub struct WebhookReqSender;

impl WebhookReqSender {
    pub fn new() -> Self {
        Self
    }

    // // dbから他サーバのデータを取得
    // async fn get_other_server(
    //     &self,
    //     ctx: &Context<'_>,
    //     dst_guild_id: u64,
    // ) -> TimesSettingCommunicatorResult<crate::other_server::OtherServer> {
    //     let other_server = ctx
    //         .data()
    //         .other_server_repository
    //         .clone()
    //         .get_from_guild_id(dst_guild_id)
    //         .await?
    //         .ok_or_else(|| anyhow::anyhow!("OtherServer not found"))?;
    //     Ok(other_server)
    // }

    // 認証局もどきから他サーバのデータを取得
    async fn get_other_server(
        &self,
        dst_guild_id: u64,
        dst_guild_name: &str,
    ) -> TimesSettingCommunicatorResult<OtherServer> {
        let ca_driver = MyCaDriver::new();

        let key_and_webhook = ca_driver.get_key_webhook(dst_guild_id).await?;

        let other_server = OtherServer::new(
            dst_guild_id,
            &dst_guild_name,
            &key_and_webhook.manage_webhook,
            &key_and_webhook.public_key_pem,
        );

        Ok(other_server)
    }

    /// webhook_urlから送信につかうWebhookを作成
    async fn get_webhook(
        &self,
        _ctx: &Context<'_>,
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
        dst_guild_name: String,
    ) -> TimesSettingCommunicatorResult<()> {
        // どのサーバに対して送信したかを記録する
        save_sent_guild_ids(ctx, dst_guild_id, dst_guild_name).await?;

        // 送信
        webhook
            .execute(&ctx, false, |w| w.content(req_message))
            .await?;

        Ok(())
    }
}

impl UbiquitimesReqSender for WebhookReqSender {
    /// 他サーバにリクエストを送信する
    ///
    /// dst_guild_idは送信先のサーバのID かならず機械的にどのサーバか特定できるもの
    /// dst_guild_nameは送信先のサーバの名前 人間が識別可能であればなんでもよい
    async fn times_setting_request_send(
        //
        &self,
        ctx: &Context<'_>,
        dst_guild_id: u64,
        dst_guild_name: &str,
        req: crate::bot_message::TimesSettingRequest,
    ) -> TimesSettingCommunicatorResult<()> {
        // 認証局もどきから他サーバのデータを取得
        let other_server = self.get_other_server(dst_guild_id, dst_guild_name).await?;
        // 送信につかうWebhookを作成
        let webhook = self.get_webhook(ctx, &other_server).await?;

        // 送信するメッセージを作成
        let req_message = self.create_req_message(ctx, other_server, req).await?;

        // メッセージをシリアライズ
        let req_message = self.serialize_req_message(req_message)?;

        let dst_guild_name = dst_guild_name.to_string();
        // 送信し，どのサーバに送信したかを記録する
        self.send_webhook(webhook, req_message, ctx, dst_guild_id, dst_guild_name)
            .await?;

        Ok(())
    }
}
