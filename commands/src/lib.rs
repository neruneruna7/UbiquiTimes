use anyhow::Result;

use poise::serenity_prelude::{self as serenity};

use serenity::{model::channel::Message, webhook::Webhook};

pub mod bot_communicate;
pub mod global_data;
pub mod master_webhook;
pub mod member_webhook;
pub mod own_server_data;

mod db_query;
pub mod sign;

use sign::claims::register_public_key_ctx_data;
use tracing::info;

use global_data::{Context, Data};
use master_webhook::MasterWebhook;
use own_server_data::ServerData;

async fn sign_str_command(ctx: &Context<'_>, enter_str: &str, sign_str: &str) -> Result<()> {
    let err_text = format!("{}と入力してください", sign_str);
    if enter_str != sign_str {
        ctx.say(&err_text).await?;
        return Err(anyhow::anyhow!(err_text));
    }

    Ok(())
}

async fn create_webhook_from_channel(
    ctx: Context<'_>,
    msg: &Message,
    name: &str,
) -> anyhow::Result<Webhook> {
    let webhook = msg.channel_id.create_webhook(ctx, name).await?;
    Ok(webhook)
}

async fn upsert_own_server_data(ctx: &Context<'_>, server_data: ServerData) -> anyhow::Result<()> {
    let connection = ctx.data().connection.clone();
    db_query::own_server_data::upsert_own_server_data(&connection, &server_data).await?;
    register_masterhook_ctx_data(&connection, ctx.data()).await?;
    Ok(())
}

pub async fn register_masterhook_ctx_data(
    connection: &sqlx::SqlitePool,
    data: &Data,
) -> anyhow::Result<()> {
    let server_data =
        db_query::own_server_data::select_own_server_data_without_guild_id(connection).await?;
    *data.master_webhook_url.write().await = server_data.master_webhook_url;
    Ok(())
}

pub async fn upsert_master_webhook(
    ctx: &Context<'_>,
    master_webhook: MasterWebhook,
) -> anyhow::Result<()> {
    db_query::master_webhooks::master_webhook_upsert(&ctx.data().connection, &master_webhook)
        .await?;
    register_public_key_ctx_data(master_webhook.guild_id, master_webhook.public_key_pem, ctx);
    Ok(())
}

async fn loged(ctx: &Context<'_>, msg: &str) -> Result<()> {
    let master_webhook_url = ctx.data().master_webhook_url.read().await;

    let webhook = Webhook::from_url(ctx, &master_webhook_url).await?;

    info!(msg);
    webhook.execute(&ctx, false, |w| w.content(msg)).await?;

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
