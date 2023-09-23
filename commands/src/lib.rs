use anyhow::Result;

use poise::serenity_prelude::{self as serenity, connection};

use serenity::{model::channel::Message, webhook::Webhook};

pub mod master_webhook;
pub mod member_webhook;
pub mod types;

mod db_query;

use tracing::info;
use types::global_data::{Context, Data};

async fn create_webhook_from_channel(
    ctx: Context<'_>,
    msg: &Message,
    name: &str,
) -> anyhow::Result<Webhook> {
    let webhook = msg.channel_id.create_webhook(ctx, name).await?;
    Ok(webhook)
}

async fn upsert_own_server_data(    
    ctx: &Context<'_>,
    server_name: &str,
    guild_id: &str,
    master_channel_id: &str,
    master_webhook_url: &str,
) -> anyhow::Result<()> {
    let connection = ctx.data().connection.clone();
    db_query::own_server_data::upsert_own_server_data(&connection, server_name, guild_id, master_channel_id, master_webhook_url).await?;
    register_masterhook_ctx_data(&connection, ctx.data()).await?;
    Ok(())
}

pub async fn register_masterhook_ctx_data(
    connection: &sqlx::SqlitePool,
    data: &Data,
) -> anyhow::Result<()> {
    let server_data = db_query::own_server_data::select_own_server_data_without_guild_id(&connection).await?;
    *data.master_webhook_url.write().await = server_data.master_webhook_url;
    Ok(())
}

async fn loged(
    ctx: &Context<'_>,
    msg: &str,
) -> Result<()> {
    let master_webhook_url = ctx.data().master_webhook_url.read().await;

    let webhook = Webhook::from_url(ctx, &master_webhook_url).await?;

    info!(msg);
    webhook.execute(&ctx, false, |w| {
        w.content(msg)
    }).await?;

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
