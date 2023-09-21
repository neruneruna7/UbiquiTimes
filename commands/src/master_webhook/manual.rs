use crate::*;

use crate::db_query::master_webhooks::*;
use crate::db_query::own_server_data::*;
use crate::types::webhook::MasterWebhook;

use anyhow::Context as anyhowContext;
use anyhow::{anyhow, Result};

use tracing::info;

/// 自身のマスターwebhook，サーバ情報を登録する
///
#[poise::command(
    prefix_command,
    track_edits,
    aliases("UTsetOwnMastereWebhook"),
    slash_command
)]
pub async fn ut_set_own_master_webhook(
    ctx: Context<'_>,
    #[description = "本サーバのサーバ名"] server_name: String,
    #[description = "本サーバのマスターwebhook URL"] master_webhook_url: String,
) -> Result<()> {
    let master_webhook = Webhook::from_url(ctx, &master_webhook_url).await?;
    let master_channel_id = master_webhook
        .channel_id
        .ok_or(anyhow!("webhookからチャンネルidを取得できませんでした"))?
        .to_string();

    let guild_id = ctx
        .guild_id()
        .ok_or(anyhow!("guild_idが取得できませんでした"))?
        .0
        .to_string();

    let connection = ctx.data().connection.clone();

    upsert_own_server_data(
        &connection,
        &server_name,
        &guild_id,
        &master_channel_id,
        &master_webhook_url,
    )
    .await?;

    ctx.say(format!("server_data: ```\n server_name: {},\n guild_id: {},\n master_channel_id: {},\n master_webhook_url: {}```", server_name, guild_id, master_channel_id, master_webhook_url)).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    track_edits,
    aliases("UTsetOtherMaster"),
    slash_command
)]
pub async fn ut_set_other_masterhook(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "拡散先サーバのギルド（サーバー）ID"] guild_id: String,
) -> Result<()> {
    // let guild_id_parsed = match guild_id {
    //     Some(id) => {
    //         let parse_result = id.parse::<u64>();
    //         match parse_result {
    //             Ok(id) => Some(id),
    //             Err(_) => {
    //                 ctx.say("guild_idは数字で指定してください。").await?;
    //                 return Ok(());
    //             }
    //         }
    //     }
    //     None => None,
    // };

    let guild_id = guild_id
        .parse::<u64>()
        .context("guild_idは数字で指定してください。")?;

    // log
    info!(
        "server_name: {}, webhook_url: {}, guild_id: {}",
        server_name, master_webhook_url, guild_id
    );

    // DBに登録する
    let connection = ctx.data().connection.clone();

    master_webhook_upsert(
        connection.as_ref(),
        MasterWebhook::from(None, &server_name, guild_id, &master_webhook_url),
    )
    .await?;

    let response_msg = format!(
        "登録しました．```\nserver_name: {}, webhook_url: {}, guild_id: {}```",
        server_name, master_webhook_url, guild_id
    );
    ctx.say(response_msg).await?;

    Ok(())
}

#[poise::command(prefix_command, track_edits, aliases("UTserverlist"), slash_command)]
pub async fn ut_serverlist(ctx: Context<'_>) -> Result<()> {
    // DBから取得する
    let connection = ctx.data().connection.clone();

    let master_webhooks = master_webhook_select_all(connection.as_ref()).await?;

    let mut response = String::new();
    for master_webhook in master_webhooks {
        response.push_str(&format!("{}\n", master_webhook.server_name));
    }

    ctx.say(response).await?;
    Ok(())
}

/// サーバ名を指定して，webhook_URLを確認する
#[poise::command(prefix_command, track_edits, aliases("UTgetMasterHook"), slash_command)]
pub async fn ut_get_master_hook(
    ctx: Context<'_>,
    #[description = "webhook_URLを確認するサーバ名"] server_name: String,
) -> Result<()> {
    // log
    info!("server_name: {}", server_name);

    // DBから取得する
    let connection = ctx.data().connection.clone();

    let master_webhook = master_webhook_select(connection.as_ref(), &server_name).await?;

    ctx.say(format!("master_webhook: {:?}", master_webhook))
        .await?;

    Ok(())
}
