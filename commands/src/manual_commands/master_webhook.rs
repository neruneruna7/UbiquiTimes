use crate::*;

use anyhow::{Result, anyhow};
use anyhow::Context as anyhowContext;


use sqlx::SqlitePool;

use tracing::info;

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
pub struct MasterWebhook {
    pub id: Option<u64>,
    pub server_name: String,
    pub guild_id: u64,
    pub webhook_url: String,
}

impl MasterWebhook {
    fn from(_id: Option<i64>, server_name: &str, guild_id: u64, webhook_url: &str) -> Self {
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
        Ok(Self {
            id: None,
            server_name: server_name.to_string(),
            guild_id: guild_id,
            webhook_url: webhook_url.to_string(),
        })
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

async fn master_webhook_insert(
    connection: &SqlitePool,
    master_webhook: MasterWebhook,
) -> anyhow::Result<()> {
    let guild_id = master_webhook.guild_id.to_string();

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
        row.guild_id.parse::<u64>()?,
        &row.webhook_url,
    );

    Ok(master_webhook)
}

// すべてのマスターwebhookを取得する
// 複数の行がとれるので、Vecに格納して返す
async fn master_webhook_select_all(connection: &SqlitePool) -> anyhow::Result<Vec<MasterWebhook>> {
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
            row.guild_id.parse::<u64>()?,
            &row.webhook_url,
        );
        master_webhooks.push(master_webhook);
    }

    Ok(master_webhooks)
}

/// 自身のマスターwebhook，サーバ情報を登録する
/// 
#[poise::command(prefix_command, track_edits, aliases("UTsetOwnMastereWebhook"), slash_command)]
pub async fn ut_set_own_master_webhook(
    ctx: Context<'_>,
    #[description = "本サーバのサーバ名"] server_name: String,
    #[description = "本サーバのマスターwebhook URL"] master_webhook_url: String,
) -> Result<()> {
    let master_webhook = Webhook::from_url(ctx, &master_webhook_url).await?;
    let master_channel_id = master_webhook
        .channel_id
        .ok_or(anyhow!("webhookからチャンネルidを取得できませんでした"))?
        .to_string();

    let guild_id = ctx.guild_id().ok_or(anyhow!("guild_idが取得できませんでした"))?.0.to_string();

    let connection = ctx.data().connection.clone();

    upsert_a_server_data(&connection, &server_name, &guild_id, &master_channel_id, &master_webhook_url).await?;

    ctx.say(format!("server_data: \n server_name: {},\n guild_id: {},\n master_channel_id: {},\n master_webhook_url: {}", server_name, guild_id, master_channel_id, master_webhook_url)).await?;

    Ok(())
}

/// 自身のマスターwebhookを a_server_data テーブルにupsertする
async fn upsert_a_server_data(
    connection: &SqlitePool,
    server_name: &str,
    guild_id: &str,
    master_channel_id: &str,
    master_webhook_url: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO a_server_data (server_name, guild_id, master_channel_id, master_webhook_url)
        VALUES(?, ?, ?, ?)
        ON CONFLICT(guild_id) DO UPDATE SET server_name = ?, master_channel_id = ?, master_webhook_url = ?;
        "#,
        server_name,
        guild_id,
        master_channel_id,
        master_webhook_url,
        server_name,
        master_channel_id,
        master_webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}



#[poise::command(prefix_command, track_edits, aliases("UTsetOtherMaster"), slash_command)]
pub async fn ut_set_other_masterhook(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "拡散先サーバのギルド（サーバー）ID"] guild_id: String,
) -> Result<()> {
    // let guild_id_parsed = match guild_id {
    //     Some(id) => {
    //         let parse_result = id.parse::<u64>();
    //         match parse_result {
    //             Ok(id) => Some(id),
    //             Err(_) => {
    //                 ctx.say("guild_idは数字で指定してください。").await?;
    //                 return Ok(());
    //             }
    //         }
    //     }
    //     None => None,
    // };

    let guild_id = guild_id.parse::<u64>().context("guild_idは数字で指定してください。")?;

    // log
    info!(
        "server_name: {}, webhook_url: {}, guild_id: {}",
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

#[poise::command(prefix_command, track_edits, aliases("UTserverlist"), slash_command)]
pub async fn ut_serverlist(ctx: Context<'_>) -> Result<()> {
    // DBから取得する
    let connection = ctx.data().connection.clone();

    let master_webhooks = master_webhook_select_all(connection.as_ref()).await?;

    let mut response = String::new();
    for master_webhook in master_webhooks {
        response.push_str(&format!("{}\n", master_webhook.server_name));
    }

    ctx.say(response).await?;
    Ok(())
}

/// サーバ名を指定して，webhook_URLを確認する
#[poise::command(prefix_command, track_edits, aliases("UTgetMasterHook"), slash_command)]
pub async fn ut_get_master_hook(
    ctx: Context<'_>,
    #[description = "webhook_URLを確認するサーバ名"] server_name: String,
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
