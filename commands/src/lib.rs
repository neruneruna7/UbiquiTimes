use anyhow::{Context as _, Result};

use poise::serenity_prelude::{self as serenity};

use serenity::{model::channel::Message, webhook::Webhook};

pub mod bot_communicate;
pub mod global_data;
pub mod other_server;
pub mod other_server_repository;
pub mod own_server;
pub mod own_server_repository;
pub mod sled_table;

mod db_query;
pub mod sign;

use sign::claims::register_public_key_ctx_data;
use tracing::info;

use global_data::{Context, Data};
use other_server::OtherServerData;
use own_server::OwnServerData;
use sled::Db;

async fn sign_str_command(ctx: &Context<'_>, enter_str: &str, sign_str: &str) -> Result<()> {
    let err_text = format!("{}と入力してください", sign_str);
    if enter_str != sign_str {
        ctx.say(&err_text).await?;
        return Err(anyhow::anyhow!(err_text));
    }

    Ok(())
}

#[allow(dead_code)]
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
    own_server_data: OwnServerData,
) -> anyhow::Result<()> {
    let db = ctx.data().connection.clone();

    own_server_data.db_upsert(db.as_ref())?;

    register_masterhook_ctx_data(&db, ctx.data()).await?;
    Ok(())
}

/// master_webhookをdbから取得しDataに登録する
pub async fn register_masterhook_ctx_data(db: &Db, data: &Data) -> anyhow::Result<()> {
    let server_data =
        OwnServerData::db_read(db)?.context("own_server_data_tableに登録されていません")?;

    *data.master_webhook_url.write().await = server_data.master_webhook_url;
    Ok(())
}

pub async fn upsert_master_webhook(
    ctx: &Context<'_>,
    other_server_data: OtherServerData,
) -> anyhow::Result<()> {
    let db = ctx.data().connection.clone();
    other_server_data.db_upsert(db.as_ref())?;

    register_public_key_ctx_data(
        other_server_data.guild_id,
        other_server_data.public_key_pem,
        ctx,
    )
    .await?;
    Ok(())
}

/// 現在エラー発生中 master_webhook_urlがdataに無いと予測
async fn logged(ctx: &Context<'_>, msg: &str) -> Result<()> {
    let master_webhook_url = ctx.data().master_webhook_url.read().await;

    let webhook = Webhook::from_url(ctx, &master_webhook_url)
        .await
        .context(format!(
            "globaldataのmaster_webhook_urlに異常があるか，登録されていません． url: {}",
            &master_webhook_url
        ))?;

    info!(msg);
    webhook.execute(&ctx, false, |w| w.content(msg)).await?;

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
