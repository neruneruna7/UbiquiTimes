use tracing::{info, error};

use crate::*;

async fn master_webhook_insert(
    connection: &SqlitePool,
    server_webhook: MasterWebhook,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO serverwebhooks (servername, webhookurl)
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
        SELECT * FROM serverwebhooks WHERE servername = ?;
        "#,
        server_name
    )
    .fetch_one(connection)
    .await?;

    let master_webhook = MasterWebhook {
        id: Some(row.id),
        server_name: row.servername,
        webhook_url: row.webhookurl,
    };

    Ok(master_webhook)
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