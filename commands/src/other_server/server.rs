use crate::*;

use crate::db_query::other_server_data::*;

use anyhow::Context as anyhowContext;
use anyhow::Result;

use tracing::info;

/// 他サーバを，拡散可能先として登録する
///
/// ここで他サーバを登録すると，メンバーはそのサーバに拡散するよう設定できるようになります．
/// まだ拡散する先として登録されたわけではありません．
#[poise::command(
    prefix_command,
    track_edits,
    aliases("UtOtherServerHook"),
    slash_command
)]
pub async fn ut_set_other_server_data(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "拡散先サーバのギルド（サーバー）ID"] guild_id: String,
    #[description = "拡散先サーバの公開鍵"] public_key_pem: String,
) -> Result<()> {
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
        &OtherServerData::new(guild_id, &server_name, &master_webhook_url, &public_key_pem),
    )
    .await?;

    let response_msg = format!(
        "登録しました．```\nserver_name: {}, webhook_url: {}, guild_id: {}```",
        server_name, master_webhook_url, guild_id
    );
    ctx.say(&response_msg).await?;

    logged(
        &ctx,
        format!("拡散可能サーバを登録しました\n{}", response_msg).as_ref(),
    )
    .await?;
    Ok(())
}

/// 拡散可能なサーバを削除する
///
/// 本サーバにおいて，拡散可能なサーバを削除します．
#[poise::command(
    prefix_command,
    track_edits,
    aliases("UtDeleteOtherServer"),
    slash_command
)]
pub async fn ut_delete_other_masterhook(
    ctx: Context<'_>,
    #[description = "削除するサーバ名"] server_name: String,
) -> Result<()> {
    // log
    info!("server_name: {}", server_name);

    // DBから削除する
    let connection = ctx.data().connection.clone();

    master_webhook_delete(connection.as_ref(), &server_name).await?;

    ctx.say(format!("{}を削除しました", server_name)).await?;

    logged(
        &ctx,
        format!("拡散可能サーバを削除しました\n{}", server_name).as_ref(),
    )
    .await?;
    Ok(())
}

/// 拡散可能なサーバ一覧
///
/// 本サーバにおいて，拡散可能なサーバの一覧を表示します．
#[poise::command(prefix_command, track_edits, aliases("UtServerlist"), slash_command)]
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
#[poise::command(prefix_command, track_edits, aliases("UtGetMasterHook"), slash_command)]
pub async fn ut_get_master_hook(
    ctx: Context<'_>,
    #[description = "webhook_URLを確認するサーバ名"] server_name: String,
) -> Result<()> {
    // log
    info!("server_name: {}", server_name);

    // DBから取得する
    let connection = ctx.data().connection.clone();

    let master_webhook =
        master_webhook_select_from_servername(connection.as_ref(), &server_name).await?;

    ctx.say(format!("master_webhook: {:?}", master_webhook))
        .await?;

    Ok(())
}
