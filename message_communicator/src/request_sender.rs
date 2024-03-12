use std::collections::HashMap;

use domain::models::communication::RequestMessage;
use domain::models::communication::TimesSettingRequest;
use domain::models::guild_data::OtherGuild;
use domain::models::guild_data::OwnGuild;
use domain::traits::communicators::GuildName;
use domain::traits::communicators::HashKey;
use domain::traits::communicators::UtReqSender;
use poise::serenity_prelude::ExecuteWebhook;
use poise::serenity_prelude::Http;
use poise::serenity_prelude::Webhook;

use domain::thiserror;

use crate::get_webhook::get_webhook;

#[derive(Debug, thiserror::Error)]
pub enum PoiseWebhookReqSenderError {
    #[error("SelenityError: {0}")]
    SelenityError(#[from] poise::serenity_prelude::Error),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub type PoiseWebhookReqSenderResult<T> = Result<T, PoiseWebhookReqSenderError>;

#[derive(Debug)]
pub struct PoiseWebhookReqSender;

impl UtReqSender for PoiseWebhookReqSender {
    type Result<T> = PoiseWebhookReqSenderResult<T>;
    /// 他サーバにリクエストを送信する
    ///
    /// dst_guild_idは送信先のサーバのID かならず機械的にどのサーバか特定できるもの
    /// dst_guild_nameは送信先のサーバの名前 人間が識別可能であればなんでもよい
    async fn times_setting_request_send(
        //
        &self,
        dst_guild: &OtherGuild,
        member_id: u64,
        req: RequestMessage,
        sent_member_and_guild_ids: &mut HashMap<HashKey, GuildName>,
    ) -> Self::Result<()> {
        // // 認証局もどきから他サーバのデータを取得
        // let other_server = self.get_other_server(dst_guild_id, dst_guild_name).await?;
        // // 送信につかうWebhookを作成
        let webhook = get_webhook(&dst_guild.webhook_url).await?;

        // // 送信するメッセージを作成
        // let req_message = self.create_req_message(ctx, other_server, req).await?;

        // メッセージをシリアライズ
        let req_message = self.serialize_req_message(req)?;

        let dst_guild_name = dst_guild.guild_name.to_string();
        // 送信し，どのサーバに送信したかを記録する
        self.send_and_record_message(
            webhook,
            req_message,
            dst_guild.guild_id,
            dst_guild_name,
            member_id,
            sent_member_and_guild_ids,
        )
        .await?;

        Ok(())
    }
}

impl PoiseWebhookReqSender {
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

    // // 認証局もどきから他サーバのデータを取得
    // async fn get_other_server(
    //     &self,
    //     dst_guild_id: u64,
    //     dst_guild_name: &str,
    // ) -> TimesSettingCommunicatorResult<OtherServer> {
    //     let ca_driver = MyCaDriver::new();

    //     let key_and_webhook = ca_driver.get_key_webhook(dst_guild_id).await?;

    //     let other_server = OtherServer::new(
    //         dst_guild_id,
    //         dst_guild_name,
    //         &key_and_webhook.manage_webhook,
    //         &key_and_webhook.public_key_pem,
    //     );

    //     Ok(other_server)
    // }

    // // 送信するメッセージを作成
    // async fn create_req_message(
    //     &self,
    //     ctx: &Context<'_>,
    //     dst_server: OtherServer,
    //     req: TimesSettingRequest,
    // ) -> TimesSettingCommunicatorResult<bot_message::RequestMessage> {
    //     let own_guild_id = ctx.guild_id().unwrap().get();

    //     let own_server_repository = ctx.data().own_server_repository.clone();
    //     let own_server = own_server_repository.get().await?;

    //     let signer = sign::UbiquitimesPrivateKey::from_pem(&own_server.private_key_pem)
    //         .context("Failed to create private key")?;

    //     let claim = Claims::from_servers_for_req(&own_server, &dst_server, req);

    //     let req_message = bot_message::RequestMessage::new(
    //         own_guild_id,
    //         dst_server.guild_id,
    //         signer.sign(claim).context("Failed to sign")?,
    //     );

    //     Ok(req_message)
    // }

    // メッセージをシリアライズ
    fn serialize_req_message(
        &self,
        req_message: RequestMessage,
    ) -> PoiseWebhookReqSenderResult<String> {
        let req_message = serde_json::to_string(&req_message)?;
        Ok(req_message)
    }

    // 送信して送信を記録する
    async fn send_and_record_message(
        &self,
        webhook: Webhook,
        req_message: String,
        dst_guild_id: u64,
        dst_guild_name: String,
        member_id: u64,
        sent_member_and_guild_ids: &mut HashMap<HashKey, GuildName>,
    ) -> PoiseWebhookReqSenderResult<()> {
        // どのサーバに対して送信したかを記録する
        Self::save_sent_guild_ids(
            sent_member_and_guild_ids,
            member_id,
            dst_guild_id,
            dst_guild_name,
        );

        let http = Http::new("");

        // 送信
        let builder = ExecuteWebhook::new().content(req_message);
        webhook.execute(&http, false, builder).await?;

        Ok(())
    }
}
