use std::sync::Arc;

use anyhow::Result;

use poise::serenity_prelude as serenity;

use serenity::{http::Http, model::channel::Message, webhook::Webhook};

use sqlx::SqlitePool;

pub mod commands;
pub mod list;

// Types used by all command functions
// すべてのコマンド関数で使用される型
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

// Dbのラッパー
pub struct Data {
    connection: Arc<SqlitePool>,
}

// TypemapKeyを実装することで、Contextに格納できるようになる
// impl TypeMapKey for UtDb {
//     type Value = Arc<SqlitePool>;
// }

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
