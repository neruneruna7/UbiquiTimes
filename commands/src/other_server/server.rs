use crate::*;

use crate::db_query::SledTable;

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

    let other_server_table = OtherServerDataTable::new(connection.as_ref());
    other_server_table
        .upsert(
            &server_name,
            &OtherServerData::new(guild_id, &server_name, &master_webhook_url, &public_key_pem),
        )
        .context("other_server_dataの登録に失敗しました")?;

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

    // dbから削除する
    let connection = ctx.data().connection.clone();

    let other_server_table = OtherServerDataTable::new(connection.as_ref());
    other_server_table
        .delete(&server_name)
        .context("other_server_dataの削除に失敗しました")?;

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

    let other_server_table = OtherServerDataTable::new(connection.as_ref());
    let other_server_data_vec = other_server_table.read_all()?;

    let mut response = String::new();
    for other_server_data in other_server_data_vec.iter() {
        response.push_str(&format!("{}\n", other_server_data.server_name));
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

    let other_server_table = OtherServerDataTable::new(connection.as_ref());
    let other_server_data =
        other_server_table
            .read(&server_name)
            .context("other_server_dataの取得に失敗しました")?;


    ctx.say(format!("master_webhook: {:?}", other_server_data.unwrap_or(OtherServerData { 
        guild_id: 0, server_name: String::from("指定したサーバ名は登録されていません"), webhook_url: String::new(), public_key_pem: String::new() 
    }).server_name))
        .await?;

    Ok(())
}
