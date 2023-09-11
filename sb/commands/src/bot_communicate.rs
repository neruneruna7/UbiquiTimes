use anyhow::{anyhow, Ok};

use crate::*;

pub struct BotMessage {
    pub src: String,
    pub dst: String,
    pub cmd: String,
    pub cmd_args: String,
    pub ttl: usize,
}

pub struct ChannelWebhookRegister {
    pub src_channel_id: i64,
    pub dst_channel_id: i64,
    pub webhook_url: String,
}

// 他サーバーから，メンバー拡散チャンネルの登録通知が来たときのしょり
async fn member_webhook_receive(
    ctx: &Context,
    msg: &Message,
    channel_webhook_register: &ChannelWebhookRegister,
) -> anyhow::Result<()> {
    let data_read = ctx.data.read().await;
    let my_server_data = data_read
        .get::<MyServerData>()
        .ok_or(anyhow!("db is None"))?
        .clone();

    let my_server_data = my_server_data.as_ref().blocking_read();

    

    Ok(())
}
