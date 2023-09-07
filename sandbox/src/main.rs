use std::env;

use serenity::async_trait;
use serenity::model::prelude::{Ready, ResumedEvent};
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{StandardFramework, CommandResult};

use tracing::{debug, error, info, instrument};
#[group]
#[commands(ping, hook, gethook)]
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
    info!("Got command '{}' by user '{}'", command_name, msg.author.name);

    true
}

#[tokio::main]
#[instrument]
async fn main() {

    tracing_subscriber::fmt::init();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    dotenvy::dotenv().ok();
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    // let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let intents = GatewayIntents::all();
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

    msg.reply(ctx, "Pong!").await?;

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
        },
        Err(e) => {
            error!("error: {:?}", e);
        }
    }

    println!("b");


    Ok(())
}

#[command]
async fn gethook(ctx: &Context, msg: &Message) -> CommandResult {

    let hooks = msg.channel_id.webhooks(&ctx.http).await?;

    msg.reply(ctx, format!("hooks: {:?}", hooks)).await?;
    Ok(())
}