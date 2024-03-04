use anyhow::{Context as _, Result};

use poise::serenity_prelude::{self as serenity};

use serenity::webhook::Webhook;

pub mod bot_message;
pub mod bot_message_communicator;
pub mod ca_driver;
pub mod global_data;
pub mod initializer;
pub mod other_server;
pub mod other_server_repository;
pub mod own_server;
pub mod own_server_repository;
pub mod poise_commands;
pub mod sled_table;

pub mod sign;

use tracing::info;

use global_data::Context;

/// 現在エラー発生中 master_webhook_urlがdataに無いと予測
// 一旦コメントアウト
async fn logged(ctx: &Context<'_>, msg: &str) -> Result<()> {
    // let master_webhook_url = ctx.data().master_webhook_url.read().await;

    // let webhook = Webhook::from_url(ctx, &master_webhook_url)
    //     .await
    //     .context(format!(
    //         "globaldataのmaster_webhook_urlに異常があるか，登録されていません． url: {}",
    //         &master_webhook_url
    //     ))?;

    // info!(msg);
    // webhook.execute(&ctx, false, |w| w.content(msg)).await?;

    Ok(())
}

/// serenityのctxだとctx.sayが使えないので
async fn logged_serenity_ctx(
    ctx: &serenity::Context,
    master_webhook_url: &str,
    msg: &str,
) -> Result<()> {
    let my_webhook = Webhook::from_url(&ctx, master_webhook_url).await?;

    info!(msg);
    my_webhook.execute(ctx, false, |w| w.content(msg)).await?;
    Ok(())
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<()> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}
