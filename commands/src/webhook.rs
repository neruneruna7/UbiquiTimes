use tracing::{error, info};

use crate::*;

#[allow(non_snake_case)]
#[command]
async fn UTregserver(ctx: &Context, msg: &Message) -> CommandResult {
    // msg.contentを分割して、server_nameとwebhook_urlを取得する
    let mut iter = msg.content.split_whitespace();
    let _ = iter.next().unwrap();
    let server_name = iter.next().unwrap();
    let guild_id = iter.next().unwrap().parse::<i64>().unwrap();
    let webhook_url = iter.next().unwrap();

    // log
    info!("server_name: {}, webhook_url: {}", server_name, webhook_url);

    // DBに登録する
    let db = get_db(ctx).await;

    match db {
        Some(db) => {
            let db = db.clone();
            master_webhook_insert(
                db.as_ref(),
                MasterWebhook {
                    id: None,
                    server_name: server_name.to_string(),
                    guild_id,
                    webhook_url: webhook_url.to_string(),
                },
            )
            .await?;
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
    let db = get_db(ctx).await.ok_or(anyhow!("db is None"))?;

    let master_webhook = master_webhook_select(db.as_ref(), server_name).await?;

    msg.reply(ctx, format!("master_webhook: {:?}", master_webhook))
        .await?;

    Ok(())
}
