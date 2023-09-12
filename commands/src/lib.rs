use std::sync::Arc;

use anyhow::Result;

use poise::serenity_prelude as serenity;

use serenity::{http::Http, model::channel::Message, webhook::Webhook};

use sqlx::SqlitePool;

#[allow(dead_code)]
pub mod list;

#[allow(dead_code)]
pub mod webhook;

// Types used by all command functions
// すべてのコマンド関数で使用される型
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

// Dbのラッパー
pub struct Data {
    connection: Arc<SqlitePool>,
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<()> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

async fn execute_ubiquitus(
    username: &str,
    content: &str,
    webhooks: Vec<String>,
) -> anyhow::Result<()> {
    // webhookを実行する
    let http = Http::new("");

    for webhook_url in webhooks.iter() {
        let webhook = Webhook::from_url(&http, webhook_url).await?;
        webhook
            .execute(&http, false, |w| w.content(content).username(username))
            .await?;
    }
    Ok(())
}

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
#[allow(dead_code)]
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

#[derive(Debug)]
#[allow(dead_code)]
// 個々人が持つwebhook
struct MemberWebhook {
    id: Option<i64>,
    server_name: String,
    member_id: i64,
    channel_id: i64,
    webhook_url: String,
}

impl MemberWebhook {
    fn from(
        _id: Option<i64>,
        server_name: &str,
        member_id: i64,
        channel_id: i64,
        webhook_url: &str,
    ) -> Self {
        Self {
            id: None,
            server_name: server_name.to_string(),
            member_id,
            channel_id,
            webhook_url: webhook_url.to_string(),
        }
    }
}

// // ContextからDbを取得する
// // poiseにより簡単に取得できるようになったので不要の可能性が高い
// // 一旦コメントアウト
// async fn get_db(ctx: Context<'_>) -> Option<Arc<SqlitePool>> {
//     let data_read = ctx.data.read().await;
//     // let data_read = ctx.data().connection.clone();
//     let db = data_read.get::<UtDb>();

//     match db {
//         Some(db) => {
//             let db = db.clone();
//             Some(db)
//         }
//         None => {
//             error!("db is None");
//             None
//         }
//     }
// }

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
#[allow(dead_code)]
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

// メンバーwebhookの登録
async fn member_webhook_insert(
    connection: &SqlitePool,
    member_webhook: MemberWebhook,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO member_webhooks (server_name, member_id, webhook_url)
        VALUES(?, ?, ?);
        "#,
        member_webhook.server_name,
        member_webhook.member_id,
        member_webhook.webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}

// メンバーwebhookの取得
#[allow(dead_code)]
async fn member_webhook_select(
    connection: &SqlitePool,
    server_name: &str,
    member_id: i64,
) -> anyhow::Result<MemberWebhook> {
    let row = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks WHERE server_name = ? AND member_id = ?;
        "#,
        server_name,
        member_id
    )
    .fetch_one(connection)
    .await?;

    let member_webhook = MemberWebhook::from(
        Some(row.id),
        &row.server_name,
        row.member_id,
        row.channel_id,
        &row.webhook_url,
    );

    Ok(member_webhook)
}

// メンバーwebhookの全取得
async fn member_webhook_select_all(
    connection: &SqlitePool,
    // server_name: &str,
    member_id: i64,
) -> anyhow::Result<Vec<MemberWebhook>> {
    let rows = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks WHERE member_id = ?;
        "#,
        member_id,
    )
    .fetch_all(connection)
    .await?;

    let mut member_webhook_list = Vec::new();
    for row in rows {
        let member_webhook = MemberWebhook::from(
            Some(row.id),
            &row.server_name,
            row.member_id,
            row.channel_id,
            &row.webhook_url,
        );
        member_webhook_list.push(member_webhook);
    }

    Ok(member_webhook_list)
}

// servername, member_idを指定してメンバーwebhookを削除する
async fn member_webhook_delete(
    connection: &SqlitePool,
    server_name: &str,
    member_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM member_webhooks WHERE server_name = ? AND member_id = ?;
        "#,
        server_name,
        member_id
    )
    .execute(connection)
    .await?;

    Ok(())
}

#[allow(dead_code)]
async fn create_webhook_from_channel(
    ctx: Context<'_>,
    msg: &Message,
    name: &str,
) -> anyhow::Result<Webhook> {
    let webhook = msg.channel_id.create_webhook(ctx, name).await?;
    Ok(webhook)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
