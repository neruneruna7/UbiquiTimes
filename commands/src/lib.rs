use std::sync::Arc;

use anyhow::anyhow;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::webhook::Webhook;
use serenity::prelude::*;

use sqlx::SqlitePool;
use tracing::error;

mod list;
mod webhook;

// Dbのラッパー
struct UtDb;

// TypemapKeyを実装することで、Contextに格納できるようになる
impl TypeMapKey for UtDb {
    type Value = Arc<SqlitePool>;
}

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
struct MasterWebhook {
    id: Option<i64>,
    server_name: String,
    guild_id: i64,
    webhook_url: String,
}

#[derive(Debug)]
// 個々人が持つwebhook
struct MemberWebhook {
    id: Option<i64>,
    server_name: String,
    user_id: i64,
    webhook_url: String,
}

// ContextからDbを取得する
async fn get_db(ctx: &Context) -> Option<Arc<SqlitePool>> {
    let data_read = ctx.data.read().await;
    let db = data_read.get::<UtDb>();

    match db {
        Some(db) => {
            let db = db.clone();
            Some(db)
        }
        None => {
            error!("db is None");
            None
        }
    }
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

    let master_webhook = MasterWebhook {
        id: Some(row.id),
        server_name: row.server_name,
        guild_id: row.guild_id,
        webhook_url: row.webhook_url,
    };

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

// メンバーwebhookの登録
async fn member_webhook_insert(
    connection: &SqlitePool,
    member_webhook: MemberWebhook,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO member_webhooks (server_name, user_id, webhook_url)
        VALUES(?, ?, ?);
        "#,
        member_webhook.server_name,
        member_webhook.user_id,
        member_webhook.webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}

// メンバーwebhookの取得
async fn member_webhook_select(
    connection: &SqlitePool,
    server_name: &str,
    user_id: i64,
) -> anyhow::Result<MemberWebhook> {
    let row = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks WHERE server_name = ? AND user_id = ?;
        "#,
        server_name,
        user_id
    )
    .fetch_one(connection)
    .await?;

    let member_webhook = MemberWebhook {
        id: Some(row.id),
        server_name: row.server_name,
        user_id: row.user_id,
        webhook_url: row.webhook_url,
    };

    Ok(member_webhook)
}

async fn create_webhook_from_channel(
    ctx: &Context,
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
