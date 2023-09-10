use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::http::Http;
use serenity::model::channel::{self, Message};
use serenity::model::prelude::{Ready, ResumedEvent};
use serenity::model::webhook::Webhook;
use serenity::prelude::*;

use sqlx::SqlitePool;
use tracing::{debug, error, info, instrument};

#[group]
#[commands(ping, pong, hook, exehook, get2hook, watch, setMasterHook, getMasterHook)]
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

    //     info!("new Message Comming!: {}", new_msg.content);
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

#[hook]
// instrument will show additional information on all the logs that happen inside
// the function.
//
// This additional information includes the function name, along with all it's arguments
// formatted with the Debug impl.
// This additional information will also only be shown if the LOG level is set to `debug`
#[instrument]
async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

// Dbのラッパー
struct UtDb;

// TypemapKeyを実装することで、Contextに格納できるようになる
impl TypeMapKey for UtDb {
    type Value = Arc<SqlitePool>;
}

#[tokio::main]
#[instrument]
async fn main() {
    tracing_subscriber::fmt::init();
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
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
        let mut pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
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
    println!("msg: {:?}", msg);

    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn pong(ctx: &Context, msg: &Message) -> CommandResult {
    // println!("ctx: {:?}", ctx.http);
    println!("channneId: {:?}", msg.channel_id);

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
async fn exehook(ctx: &Context, msg: &Message) -> CommandResult {
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

// 特定のチャンネルの投稿を監視する.
#[command]
async fn watch(ctx: &Context, msg: &Message) -> CommandResult {
    let channel_id = msg.channel_id;
    let mut stream = channel_id
        .messages(ctx, |retriever| retriever.limit(10))
        .await?;

    for message in stream.iter() {
        println!("message: {:?}", message.content);
    }

    Ok(())
}

// // キーバリューストアにwebhook URLを格納する
// #[command]
// async fn setwebhook(ctx: &Context, _msg: &Message) -> CommandResult {
//     let v = vec!["a", "we", "we", "afgr"]
//         .iter()
//         .map(|s| s.to_string())
//         .collect::<Vec<String>>();

//     let key = "webhook_url";

//     let data_read = ctx.data.read().await;
//     let db = data_read
//         .get::<UtDb>()
//         .expect("Expect UtDb in typemap")
//         .clone();

//     db.my_set(key, &v)?;

//     Ok(())
// }

// #[command]
// async fn getwebhook(ctx: &Context, msg: &Message) -> CommandResult {
//     let data_read = ctx.data.read().await;
//     let db = data_read
//         .get::<UtDb>()
//         .expect("Expect UtDb in typemap")
//         .clone();

//     let key = "webhook_url";

//     // let result = db.get(&key).unwrap();
//     // let ivec = result.unwrap();
//     // let string_value = String::from_utf8(ivec.to_vec())?;

//     // println!("{}", string_value);

//     let webhook_urls = EzKvs::<Vec<String>>::my_get(db.as_ref(), key)?;

//     msg.reply(ctx, format!("webhook_urls: {:?}", webhook_urls))
//         .await?;
//     Ok(())
// }

// // msg contentの内容をkvsに保存する
// #[command]
// async fn savemsg(ctx: &Context, msg: &Message) -> CommandResult {
//     let data_read = ctx.data.read().await;
//     let db = data_read
//         .get::<UtDb>()
//         .expect("Expect UtDb in typemap")
//         .clone();

//     let key = "msg";

//     db.my_set(key, &msg.content)?;

//     println!("msg: {:?}", &msg.content);

//     msg.reply(ctx, format!("msg: {}", msg.content)).await?;
//     Ok(())
// }

// // kvsに保存したmsg contentを取得する
// #[command]
// async fn getmsg(ctx: &Context, msg: &Message) -> CommandResult {
//     let data_read = ctx.data.read().await;
//     let db = data_read
//         .get::<UtDb>()
//         .expect("Expect UtDb in typemap")
//         .clone();

//     let key = "msg";

//     let mut msg_content =
//         EzKvs::<String>::my_get(db.as_ref(), key)?.unwrap_or("msg not found".to_string());

//     // \nを\r\nに置換する
//     // msg_content = msg_content.replace("\n", r#"\r\n"#);

//     println!("msg_content: {:?}", &msg_content);

//     msg.reply(ctx, format!("msg: {:?}", msg_content)).await?;
//     Ok(())
// }

// fn add_webhook(db: &Db, webhook_url: &str) {
//     // let db = sled::open("./db").unwrap();

//     let key = "test2";

//     let mut webhooks = EzKvs::<Vec<String>>::my_get(db, key).unwrap().unwrap();
//     webhooks.push(webhook_url.to_string());

//     EzKvs::<Vec<String>>::my_set(db, key, &webhooks).unwrap();

//     info!("execute add webhook: {}", webhook_url);
// }

async fn sqlx_test(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    // let row = sqlx::query("SELECT * FROM users WHERE id = ?")
    //     .bind(1)
    //     .fetch_one(&mut tx)
    //     .await?;

    // let id: i32 = row.try_get("id")?;
    // let name: String = row.try_get("name")?;

    // println!("id: {}, name: {}", id, name);

    tx.commit().await?;

    Ok(())
}

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
struct MasterWebhook {
    id: Option<i64>,
    server_name: String,
    // guild_id: i64,
    webhook_url: String,
}

#[derive(Debug)]
// 個々人が持つwebhook
struct PrivateWebhook {
    id: Option<i64>,
    server_name: String,
    user_id: i64,
    webhook_url: String,
}

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
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    // #[test]
    // fn it_works() {
    //     let db = sled::open("../../db").unwrap();
    //     test_ezkvs(&db);
    //     test_add_webhook(&db);
    // }

    // fn test_ezkvs(db: &Db) {
    //     let a = vec![1, 2];
    //     db.my_set("test", &a).unwrap();
    //     let b = EzKvs::<Vec<i32>>::my_get(db, "test").unwrap();
    //     let _ = db.drop_tree("test");
    //     assert_eq!(a, b.unwrap());
    // }

    // // #[test]
    // fn test_add_webhook(db: &Db) {
    //     // let db = sled::open("./db").unwrap();
    //     let a = Iterator::collect::<Vec<String>>(vec!["1", "2"].iter().map(|x| x.to_string()));

    //     db.my_set("test2", &a).unwrap();
    //     add_webhook(&db, "3");
    //     let b = EzKvs::<Vec<String>>::my_get(db, "test2").unwrap();

    //     let c = vec!["1", "2", "3"];
    //     let c = c.iter().map(|x| x.to_string()).collect::<Vec<String>>();
    //     let _ = db.drop_tree("test2");
    //     assert_eq!(c, b.unwrap());
    // }

    use sqlx::{
        sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
        ConnectOptions, Connection, Row, Sqlite, SqlitePool, Transaction,
    };

    #[tokio::test]
    async fn sqlite_te() {
        let database_url = "sqlite:db/ubiquitimes.db";
        let pool = SqlitePool::connect(database_url).await.unwrap();

        let mut conn = pool.acquire().await.unwrap();

        let a = conn
            .transaction(|txn| {
                Box::pin(async move { sqlx::query("select * from ..").fetch_all(&mut **txn).await })
            })
            .await;

        // // コネクションの設定
        // let connection_options = SqliteConnectOptions::try_from(database_url).unwrap()
        //     // DBが存在しないなら作成する
        //     .create_if_missing(true)
        //     // トランザクション使用時の性能向上のため、WALを使用する らしい
        //     .journal_mode(SqliteJournalMode::Wal)
        //     .synchronous(SqliteSynchronous::Normal);

        // // 上の設定を使ってコネクションプールを作成する
        // let sqlite_pool = SqlitePoolOptions::new()
        //     .connect_with(connection_options)
        //     .await
        //     .unwrap();
    }
}
