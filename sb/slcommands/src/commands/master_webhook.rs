use crate::*;

use anyhow::Result;

use poise::serenity_prelude::{self as serenity, guild};

use sqlx::SqlitePool;

use tracing::info;

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
pub struct MasterWebhook {
    pub id: Option<u64>,
    pub server_name: String,
    pub guild_id: Option<u64>,
    pub webhook_url: String,
}

impl MasterWebhook {
    fn from(_id: Option<i64>, server_name: &str, guild_id: Option<u64>, webhook_url: &str) -> Self {
        Self {
            id: None,
            server_name: server_name.to_string(),
            guild_id,
            webhook_url: webhook_url.to_string(),
        }
    }

    fn from_row(
        _id: Option<i64>,
        server_name: &str,
        guild_id: &str,
        webhook_url: &str,
    ) -> Result<Self> {

        let guild_id = guild_id.parse::<u64>()?;
        Ok(
        Self {
            id: None,
            server_name: server_name.to_string(),
            guild_id: Some(guild_id),
            webhook_url: webhook_url.to_string(),
        }
    )
    }

}

// // bot間通信に関わるコマンドの種類
// // 通信の際に必要なデータはここに内包する
// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// enum CmdKind {
//     MemberWebhookAutoRegister,
//     MemberWebhookAutoRegisterResponse,
// }

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// struct BotComMessage {
//     src_server: String,
//     dst_server: String,
//     cmd_kind: CmdKind,
//     ttl: i64,
//     timestamp: chrono::DateTime<chrono::Utc>,
// }

// struct MemberWebhookAutoRegisters {
//     member_id: u64,
//     channel_id: u64,
//     webhook_url: String,
// }

pub async fn master_webhook_insert(
    connection: &SqlitePool,
    master_webhook: MasterWebhook,
) -> anyhow::Result<()> {
    let guild_id = master_webhook.guild_id.unwrap().to_string();

    sqlx::query!(
        r#"
        INSERT INTO master_webhooks (server_name, guild_id, webhook_url)
        VALUES(?, ?, ?);
        "#,
        master_webhook.server_name,
        guild_id,
        master_webhook.webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn master_webhook_select(
    connection: &SqlitePool,
    server_name: &str,
) -> anyhow::Result<MasterWebhook> {
    let row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks WHERE server_name = ?;
        "#,
        server_name
    )
    .fetch_one(connection)
    .await?;

    let master_webhook = MasterWebhook::from(
        Some(row.id),
        &row.server_name,
        Some(row.guild_id.parse::<u64>()?),
        &row.webhook_url,
    );

    Ok(master_webhook)
}

// すべてのマスターwebhookを取得する
// 複数の行がとれるので、Vecに格納して返す
pub async fn master_webhook_select_all(
    connection: &SqlitePool,
) -> anyhow::Result<Vec<MasterWebhook>> {
    let rows = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks;
        "#,
    )
    .fetch_all(connection)
    .await?;

    let mut master_webhooks = Vec::new();

    for row in rows {
        let master_webhook = MasterWebhook::from(
            Some(row.id),
            &row.server_name,
            Some(row.guild_id.parse::<u64>()?),
            &row.webhook_url,
        );
        master_webhooks.push(master_webhook);
    }

    Ok(master_webhooks)
}


#[poise::command(prefix_command, track_edits, aliases("UTregMaster"), slash_command)]
pub async fn ut_masterhook_register(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "拡散先サーバのギルド（サーバー）ID"] guild_id: Option<u64>,
) -> Result<()> {
    // msg.contentを分割して、server_nameとwebhook_urlを取得する
    // let mut iter = msg.content.split_whitespace();
    // let _ = iter.next().unwrap();
    // let server_name = iter.next().unwrap();
    // let guild_id = iter.next().unwrap().parse::<i64>().unwrap();
    // let webhook_url = iter.next().unwrap();

    // log
    info!(
        "server_name: {}, webhook_url: {}, guild_id: {:?}",
        server_name, master_webhook_url, guild_id
    );

    // DBに登録する
    let connection = ctx.data().connection.clone();

    master_webhook_insert(
        connection.as_ref(),
        MasterWebhook::from(None, &server_name, guild_id, &master_webhook_url),
    )
    .await?;


    Ok(())
}

#[poise::command(prefix_command, track_edits, aliases("UTgetMasterHook"), slash_command)]
pub async fn get_master_hook(
    ctx: Context<'_>,
    #[description = "webhookを確認するサーバ名"] server_name: String,
) -> Result<()> {
    // log
    info!("server_name: {}", server_name);

    // DBから取得する
    let connection = ctx.data().connection.clone();

    let master_webhook = master_webhook_select(connection.as_ref(), &server_name).await?;

    ctx.say(format!("master_webhook: {:?}", master_webhook))
        .await?;

    Ok(())
}

