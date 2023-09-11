use serenity::model::webhook::Webhook;
use tracing::{info, error};

use crate::*;

async fn master_webhook_insert(
    connection: &SqlitePool,
    server_webhook: MasterWebhook,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO master_webhooks (server_name, webhook_url)
        VALUES(?, ?);
        "#,
        server_webhook.server_name,
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
        webhook_url: row.webhook_url,
    };

    Ok(master_webhook)
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

#[allow(non_snake_case)]
#[command]
async fn setMasterHook(ctx: &Context, msg: &Message) -> CommandResult {
    // msg.contentを分割して、server_nameとwebhook_urlを取得する
    let mut iter = msg.content.split_whitespace();
    let _ = iter.next().unwrap();
    let server_name = iter.next().unwrap();
    let webhook_url = iter.next().unwrap();

    // log
    info!("server_name: {}, webhook_url: {}", server_name, webhook_url);

    // DBに登録する
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<UtDb>();

    match db {
        Some(db) => {
            let db = db.clone();
            master_webhook_insert(db.as_ref(), MasterWebhook {
                id: None,
                server_name: server_name.to_string(),
                webhook_url: webhook_url.to_string(),
            }).await?;
        }
        None => {
            error!("db is None");
            msg.reply(ctx, "[error] db is None").await?;
        }
    }
    
    Ok(())
}

#[allow(non_snake_case)]
#[command]
async fn getMasterHook(ctx: &Context, msg: &Message) -> CommandResult {
    // msg.contentを分割して、server_nameを取得する
    let mut iter = msg.content.split_whitespace();
    let _ = iter.next().unwrap();
    let server_name = iter.next().unwrap();

    // log
    info!("server_name: {}", server_name);

    // DBから取得する
    let data_read = ctx.data.read().await;
    let db = data_read
        .get::<UtDb>()
        .expect("Expect UtDb in typemap")
        .clone();

    let master_webhook = master_webhook_select(db.as_ref(), server_name).await?;

    msg.reply(ctx, format!("master_webhook: {:?}", master_webhook))
        .await?;

    Ok(())
}

async fn create_webhook_from_channel(
    ctx: &Context,
    msg: &Message,
    name: &str,
) -> anyhow::Result<Webhook> {
    let webhook = msg.channel_id.create_webhook(ctx, name).await?;
    Ok(webhook)
}