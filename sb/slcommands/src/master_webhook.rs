use crate::*;

use anyhow::Result;

use poise::serenity_prelude as serenity;

use serenity::{http::Http, model::channel::Message, webhook::Webhook};

use sqlx::SqlitePool;

use tracing::info;

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
struct MasterWebhook {
    id: Option<i64>,
    server_name: String,
    guild_id: Option<i64>,
    webhook_url: String,
}

impl MasterWebhook {
    fn from(_id: Option<i64>, server_name: &str, guild_id: Option<i64>, webhook_url: &str) -> Self {
        Self {
            id: None,
            server_name: server_name.to_string(),
            guild_id,
            webhook_url: webhook_url.to_string(),
        }
    }
}

// bot間通信に関わるコマンドの種類
// 通信の際に必要なデータはここに内包する
#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum CmdKind {
    MemberWebhookAutoRegister,
    MemberWebhookAutoRegisterResponse,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct BotComMessage {
    src_server: String,
    dst_server: String,
    cmd_kind: CmdKind,
    ttl: i64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

struct MemberWebhookAutoRegisters {
    member_id: u64,
    channel_id: u64,
    webhook_url: String,
}

async fn master_webhook_insert(
    connection: &SqlitePool,
    server_webhook: MasterWebhook,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO master_webhooks (server_name, guild_id, webhook_url)
        VALUES(?, ?, ?);
        "#,
        server_webhook.server_name,
        server_webhook.guild_id,
        server_webhook.webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}

async fn master_webhook_select(
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
        Some(row.guild_id),
        &row.webhook_url,
    );

    Ok(master_webhook)
}

// すべてのマスターwebhookを取得する
// 複数の行がとれるので、Vecに格納して返す
async fn master_webhook_select_all(
    connection: &SqlitePool,
    _server_name: &str,
) -> anyhow::Result<()> {
    let _row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks;
        "#,
    )
    .fetch_one(connection)
    .await?;

    // let master_webhook = MasterWebhook {
    //     id: Some(row.id),
    //     server_name: row.server_name,
    //     webhook_url: row.webhook_url,
    // };

    // Ok(master_webhook)

    Ok(())
}


#[poise::command(prefix_command, track_edits, aliases("UTregMaster"), slash_command)]
pub async fn ut_masterhook_register(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "拡散先サーバのギルド（サーバー）ID"] guild_id: Option<i64>,
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

