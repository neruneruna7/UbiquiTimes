use std::env;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::http::Http;
use serenity::model::channel::{Message, self};
use serenity::model::prelude::{Ready, ResumedEvent};
use serenity::model::webhook::Webhook;
use serenity::prelude::*;

use sled::Db;

use tracing::{debug, error, info, instrument};

#[group]
#[commands(ping, pong, hook,exehook ,get2hook, setwebhook, getwebhook, savemsg, getmsg, watch)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        // Log at the INFO level. This is a macro from the `tracing` crate.
        info!("{} is connected!", ready.user.name);
    }

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

trait EzKvs<T: serde::Serialize + for<'a> serde::Deserialize<'a>> {
    fn my_set(&self, key: &str, value: &T) -> anyhow::Result<()>;
    fn my_get(&self, key: &str) -> anyhow::Result<Option<T>>;
}

impl<T: serde::Serialize + for<'a> serde::Deserialize<'a>> EzKvs<T> for Db {
    fn my_set(&self, key: &str, value: &T) -> anyhow::Result<()> {
        let json = serde_json::to_string(&value).unwrap();
        // let key = key.as_bytes();
        let value = json.as_bytes();

        self.insert(key, value)?;

        Ok(())
    }

    fn my_get(&self, key: &str) -> anyhow::Result<Option<T>> {
        // let key = key.as_bytes();
        let result = self.get(key)?;
        if let Some(ivec) = result {
            let string_value = String::from_utf8(ivec.to_vec())?;
            let json = serde_json::from_str(string_value.as_str())?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }
}

// fn my_get<T: for<'a> serde::Deserialize<'a>>(db: &Db, key: &str) -> anyhow::Result<Option<T>> {
//     // let key = key.as_bytes();
//     let result = db.get(key)?;
//     if let Some(ivec) = result {
//         let string_value = String::from_utf8(ivec.to_vec())?;
//         let json = serde_json::from_str(string_value.as_str())?;
//         Ok(Some(json))
//     } else {
//         Ok(None)
//     }
// }

// fn my_set(db: &Db, key: &str, value: &impl serde::Serialize) -> anyhow::Result<()> {
//     let json = serde_json::to_string(&value)?;
//     // let key = key.as_bytes();
//     let value = json.as_bytes();

//     db.insert(key, value)?;

//     Ok(())
// }

// キーバリューストアにwebhook URLを格納する
#[command]
async fn setwebhook(_ctx: &Context, _msg: &Message) -> CommandResult {
    let v = vec!["a", "we", "we", "afgr"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let key = "webhook_url";

    let db = sled::open("./db").unwrap();

    db.my_set(key, &v)?;
    Ok(())
}

#[command]
async fn getwebhook(ctx: &Context, msg: &Message) -> CommandResult {
    let db = sled::open("./db").unwrap();

    let key = "webhook_url";

    // let result = db.get(&key).unwrap();
    // let ivec = result.unwrap();
    // let string_value = String::from_utf8(ivec.to_vec())?;

    // println!("{}", string_value);

    let webhook_urls = EzKvs::<Vec<String>>::my_get(&db, key)?;

    msg.reply(ctx, format!("webhook_urls: {:?}", webhook_urls))
        .await?;
    Ok(())
}

// msg contentの内容をkvsに保存する
#[command]
async fn savemsg(ctx: &Context, msg: &Message) -> CommandResult {
    let db = sled::open("./db").unwrap();

    let key = "msg";

    db.my_set(key, &msg.content)?;

    println!("msg: {:?}", &msg.content);

    msg.reply(ctx, format!("msg: {}", msg.content)).await?;
    Ok(())
}

// kvsに保存したmsg contentを取得する
#[command]
async fn getmsg(ctx: &Context, msg: &Message) -> CommandResult {
    let db = sled::open("./db").unwrap();

    let key = "msg";

    let mut msg_content = EzKvs::<String>::my_get(&db, key)?.unwrap_or("msg not found".to_string());

    // \nを\r\nに置換する
    // msg_content = msg_content.replace("\n", r#"\r\n"#);

    println!("msg_content: {:?}", &msg_content);

    msg.reply(ctx, format!("msg: {:?}", msg_content)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let db = sled::open("./db").unwrap();
        let a = vec![1, 2];
        db.my_set("test", &a).unwrap();
        let b = EzKvs::<Vec<i32>>::my_get(&db, "test").unwrap();
        let _ = db.drop_tree("test");
        assert_eq!(a, b.unwrap());
    }
}
