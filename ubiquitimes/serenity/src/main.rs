use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{CommandError, CommandResult, StandardFramework};
use serenity::http::Http;
use serenity::model::channel::{Message};
use serenity::model::prelude::{Ready, ResumedEvent};
use serenity::model::webhook::Webhook;
use serenity::prelude::*;

use sqlx::SqlitePool;
use tracing::{debug, error, info, instrument};

#[group]
#[commands(ping)]
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
// #[instrument]
async fn before_hook(_: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

// エラーをログに表示
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
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
