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
                MasterWebhook::from(
                    None,
                    server_name,
                    guild_id,
                    webhook_url,)
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

// メンバーwebhookを登録する
#[allow(non_snake_case)]
#[command]
async fn UTregister(ctx: &Context, msg: &Message) -> CommandResult {
    // msg.contentを分割して、server_nameとチャンネルidを取得する
    let mut iter = msg.content.split_whitespace();
    let _ = iter.next().unwrap();
    let server_name = iter.next().unwrap();
    let channel_id = iter.next().unwrap().parse::<i64>().unwrap();


    // もしチャンネルにwebhookが存在していたら、それを使う
    // なければ、新規に作成する
    // チャンネルidから，存在しているwebhookを取得する
    let webhooks = msg.channel_id.webhooks(&ctx).await?;
    
    // UT- username という名前のwebhookがあるかどうか
    let webhook = if let Some(webhook) = webhooks.iter().find(|w| w.name == Some(format!("UT-{}", &msg.author.name))) {
        webhook.to_owned()
    } else {
        msg.channel_id.create_webhook(&ctx, format!("UT-{}", &msg.author.name)).await?
    };

    let my_webhook_url = webhook.url()?;

    // さらなる記述が必要

    Ok(())
}


// 手動でメンバーwebhookを登録する
// (prefix)UTregisterM server_name webhook_url 
#[allow(non_snake_case)]
#[command]
async fn UTregisterM(ctx: &Context, msg: &Message) -> CommandResult {
    // msg.contentを分割して、server_nameとチャンネルidを取得する
    let mut iter = msg.content.split_whitespace();
    let _ = iter.next().unwrap();
    let server_name = iter.next().unwrap().to_string();
    let webhook_url = iter.next().unwrap().to_string();

    let db = get_db(ctx).await.unwrap();

    let member_webhook = MemberWebhook::from(
        None,
        &server_name,
        msg.author.id.0 as i64,
        &webhook_url,
    );

    member_webhook_insert(db.as_ref(), member_webhook).await?;

    info!("member webhook inserted");
    msg.reply(ctx, "member webhook registed").await?;

    Ok(())
}

// メンバーwebhookを取得して，リプライにリスト表示する
#[allow(non_snake_case)]
#[command]
async fn UTlist(ctx: &Context, msg: &Message) -> CommandResult {
    let db = get_db(ctx).await.unwrap();

    // SqliteのINTEGER型はi64になる都合で，i64に変換する
    // discordのidは18桁で構成されており，i64に収まるため変換しても問題ないと判断した
    let member_id = msg.author.id.0 as i64;

    let member_webhooks = member_webhook_select_all(db.as_ref(), member_id).await?;

    let mut reply = String::new();

    for member_webhook in member_webhooks {
        reply.push_str(&format!("{}\n", member_webhook.server_name));
    }

    msg.reply(ctx, reply).await?;

    Ok(())
}


// メンバーwebhookを削除する
#[allow(non_snake_case)]
#[command]
async fn UTdelete(ctx: &Context, msg: &Message) -> CommandResult {
    // msg.contentを分割して、server_nameを取得する
    let mut iter = msg.content.split_whitespace();
    let _ = iter.next().unwrap();
    let server_name = iter.next().unwrap();

    let db = get_db(ctx).await.unwrap();

    // SqliteのINTEGER型はi64になる都合で，i64に変換する
    // discordのidは18桁で構成されており，i64に収まるため変換しても問題ないと判断した
    let member_id = msg.author.id.0 as i64;

    member_webhook_delete(db.as_ref(), server_name, member_id).await?;

    info!("member webhook deleted");
    msg.reply(ctx, "member webhook deleted").await?;

    Ok(())
}

// メンバーwebhookにたいして，拡散する
#[allow(non_snake_case)]
#[command]
async fn UT(ctx: &Context, msg: &Message) -> CommandResult {
    // 文字列からコマンド名を削除する
    let content = msg.content.clone();
    let content = content.replace("~UT", "");

    // let mut iter = msg.content.splitn(2, ' ');
    // let _ = iter.next().unwrap();
    // let content = iter.next().unwrap();

    let username = format!("UT-{}", msg.author.name.as_str());


    // DBからそのユーザのwebhookをすべて取得する
    let db = get_db(ctx).await.unwrap();

    // SqliteのINTEGER型はi64になる都合で，i64に変換する
    // discordのidは18桁で構成されており，i64に収まるため変換しても問題ないと判断した
    let member_id = msg.author.id.0 as i64;
    let member_webhooks = member_webhook_select_all(db.as_ref(), member_id).await?;

    let member_webhooks = member_webhooks.iter().map(|m| m.webhook_url.to_owned()).collect::<Vec<String>>();

    execute_ubiquitus(&username, &content, member_webhooks).await?;

    Ok(())
}

