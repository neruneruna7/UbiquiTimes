pub mod error;
pub mod request_receiver;
pub mod request_sender;
pub mod response_receiver;

pub(crate) mod get_webhook {
    use poise::serenity_prelude::{Http, Webhook};

    /// webhook_urlから送信につかうWebhookを作成
    pub(crate) async fn get_webhook(webhook_url: &str) -> poise::serenity_prelude::Result<Webhook> {
        // 送信だけなら特にトークン無しでもいいらしい
        // むしろトークン次第で何ができるのか気になるところ
        let http = Http::new("");
        let webhook = Webhook::from_url(http, webhook_url).await?;
        Ok(webhook)
    }
}
