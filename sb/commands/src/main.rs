use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{CommandError, CommandResult, StandardFramework};
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::prelude::{Ready, ResumedEvent};
use serenity::model::webhook::Webhook;
use serenity::prelude::*;

use sqlx::SqlitePool;
use tracing::{debug, error, info, instrument};

mod bot_communicate;
mod webhook;

use crate::webhook::*;

#[group]
#[commands(ping, pong, hook, exehook, get2hook, sqlxtest, UTregisterM, UTlist, UT, UTdelete)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        // Log at the INFO level. This is a macro from the `tracing` crate.
        info!("{} is connected!", ready.user.name);
    }

    // // 特定のチャンネルへの投稿を監視する
    // async fn message(&self, _ctx: Context, new_msg: Message) {
    //     if new_msg.channel_id.0 != 1002902939716812873 || new_msg.author.id == 1147398117717704714 {
    //         return;
    //     }

    //     info!("new Message Coming!: {}", new_msg.content);
    //     new_msg.reply(&_ctx, "hello!").await.unwrap();
    // }

    // For instrument to work, all parameters must implement Debug.
    //
    // Handler doesn't implement Debug here, so we specify to skip that argument.
    // Context doesn't implement Debug either, so it is also skipped.
    #[instrument(skip(self, _ctx))]
    async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
        // Log at the DEBUG level.
        //
        // In this example, this will not show up in the logs because DEBUG is
        // below INFO, which is the set debug level.
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

// このbeforeの使い方がわからない
//
// この関数はコマンドが実行される前に実行される
// この関数の戻り値がtrueの場合、コマンドが実行される

#[hook]
// instrument will show additional information on all the logs that happen inside
// the function.
//
// This additional information includes the function name, along with all it's arguments
// formatted with the Debug impl.
// This additional information will also only be shown if the LOG level is set to `debug`
// #[instrument]
async fn before_hook(_: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

// データベース関連の処理はここにまとめる

async fn after_hook_db(ctx: &Context, msg: &Message, command_name: &str) -> CommandResult {

    Ok(())
}

#[hook]
async fn after_hook(_: &Context, _: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    //  Print out an error if it happened
    if let Err(why) = error {
        println!("Error in {}: {:?}", cmd_name, why);
    }
}

// Dbのラッパー
struct UtDb;

// TypemapKeyを実装することで、Contextに格納できるようになる
impl TypeMapKey for UtDb {
    type Value = Arc<SqlitePool>;
}

// 自身のマスターwebhook, botComチャンネルのwebhookとid
struct MyServerData {
    master_webhook: Option<String>,
    botcom_channel_id: Option<i64>,
    botcom_webhook_id: Option<i64>,
}

impl TypeMapKey for MyServerData {
    type Value = Arc<RwLock<MyServerData>>;
}

#[tokio::main]
// #[instrument]
async fn main() {
    tracing_subscriber::fmt::init();
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .before(before_hook)
        .after(after_hook)
        .group(&GENERAL_GROUP);

    dotenvy::dotenv().ok();
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    // let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_WEBHOOKS;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();
        let mut data = client.data.write().await;
        data.insert::<UtDb>(Arc::new(pool));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    // println!("ctx: {:?}", ctx.http);
    // println!("msg: {:?}", msg);

    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn pong(ctx: &Context, msg: &Message) -> CommandResult {
    // println!("ctx: {:?}", ctx.http);
    // println!("channneId: {:?}", msg.channel_id);

    msg.reply(ctx, &msg.content).await?;

    Ok(())
}

#[command]
async fn hook(ctx: &Context, msg: &Message) -> CommandResult {
    // 送り元のチャンネルへのwebhookを作成
    println!("a");

    let webhook = msg.channel_id.create_webhook(&ctx, "testbot").await;

    match webhook {
        Ok(webhook) => {
            println!("webhook: {:?}", webhook);
        }
        Err(e) => {
            error!("error: {:?}", e);
        }
    }

    println!("b");

    Ok(())
}

// webhookを実行
#[command]
async fn exehook(_ctx: &Context, _msg: &Message) -> CommandResult {
    // let webhook_url = "https://discord.com/api/webhooks";

    // You don't need a token when you are only dealing with webhooks.
    let http = Http::new("");
    let webhook = Webhook::from_url(
        &http,
        "https://discord.com/api/webhooks/1148062413812404244/W2xVsl1Jt055ovjr8KRzV9zoDW3UPJcGhoTMGzLk6dPZJKNhLRDAodh3TOYyYnjSwFjc"
    )
        .await
        .expect("Replace the webhook with your own");

    webhook
        .execute(&http, false, |w| {
            w.content("@di-bot-rust test webhook")
                .username("Webhook test")
        })
        .await
        .expect("Could not execute webhook.");

    Ok(())
}

#[command]
async fn get2hook(ctx: &Context, msg: &Message) -> CommandResult {
    let hooks = msg.channel_id.webhooks(&ctx.http).await?;

    msg.reply(ctx, format!("hooks: {:?}", hooks)).await?;
    Ok(())
}


#[command]
async fn sqlxtest(ctx: &Context, _msg: &Message) -> CommandResult {
    println!("hey");
    // DBから取得する
    let db = get_db(ctx).await.ok_or(anyhow::anyhow!("db is None"))?;

    let _masterwebhook1 = MasterWebhook {
        id: None,
        server_name: "test1".to_string(),
        guild_id: 1,
        webhook_url: "https://discord.com/api/webhooks/1148062413812404244/W2xVsl1Jt055ovjr8KRzV9zoDW3UPJcGhoTMGzLk6dPZJKNhLRDAodh3TOYyYnjSwFjc".to_string(),
    };

    let _masterwebhook2 = MasterWebhook {
        id: None,
        server_name: "test2".to_string(),
        guild_id: 2,
        webhook_url: "https://discord.com/api/webhooks/1148062413812404244/W2xVsl1Jt055ovjr8KRzV9zoDW3UPJcGhoTMGzLk6dPZJKNhLRDAodh3TOYyYnjSwFjc".to_string(),
    };

    // master_webhook_insert(db.as_ref(), masterwebhook1).await?;
    // println!("hey");

    // master_webhook_insert(db.as_ref(), masterwebhook2).await?;
    // println!("hey");

    let row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks;
        "#
    )
    .fetch_all(db.as_ref())
    .await?;

    let mut reply = String::new();
    for row in row {
        reply.push_str(&format!("{}\n", row.server_name));
    }

    println!("row: {}", reply);
    // let mut reply = String::new();
    // for master_webhook in master_webhook_list {
    //     reply.push_str(&format!("{}\n", master_webhook.server_name));
    // }

    // Ok(())
    Err("hey".into())
}


use anyhow::anyhow;
use serenity::prelude::*;
async fn execute_ubiquitus(username: &str, content: &str, webhooks: Vec<String>) -> anyhow::Result<()> {
    // webhookを実行する
    let http = Http::new("");

    for webhook_url in webhooks.iter() {
        let webhook = Webhook::from_url(&http, webhook_url).await?;
        webhook
        .execute(&http, false, |w| {
            w.content(content)
                .username(username)
        })
        .await?;

    }
    Ok(())
}

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
struct MasterWebhook {
    id: Option<i64>,
    server_name: String,
    guild_id: i64,
    webhook_url: String,
}

impl MasterWebhook {
    fn from (id: Option<i64>, server_name: &str, guild_id: i64, webhook_url: &str) -> Self {
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
    webhook_url: String,
}

impl MemberWebhook {
    fn from (id: Option<i64>, server_name: &str, member_id: i64, webhook_url: &str) -> Self {
        Self {
            id: None,
            server_name: server_name.to_string(),
            member_id,
            webhook_url: webhook_url.to_string(),
        }
    }
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

    let master_webhook = MasterWebhook::from(Some(row.id), &row.server_name, row.guild_id, &row.webhook_url);

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

    let member_webhook = MemberWebhook::from(Some(row.id), &row.server_name, row.member_id, &row.webhook_url);

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
        let member_webhook = MemberWebhook::from(Some(row.id), &row.server_name, row.member_id, &row.webhook_url);
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

async fn create_webhook_from_channel(
    ctx: &Context,
    msg: &Message,
    name: &str,
) -> anyhow::Result<Webhook> {
    let webhook = msg.channel_id.create_webhook(ctx, name).await?;
    Ok(webhook)
}


#[cfg(test)]
mod tests {

    use sqlx::{ConnectOptions, Connection, SqlitePool};

    #[tokio::test]
    async fn sqlite_te() {
        let database_url = "sqlite:db/ubiquitimes.db";
        let pool = SqlitePool::connect(database_url).await.unwrap();

        let mut conn = pool.acquire().await.unwrap();

        let _a = conn
            .transaction(|txn| {
                Box::pin(async move { sqlx::query("select * from ..").fetch_all(&mut **txn).await })
            })
            .await;
    }
}
