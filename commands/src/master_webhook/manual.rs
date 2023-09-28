use crate::*;

use crate::db_query::master_webhooks::*;
use crate::sign::key_gen::*;
use crate::types::webhook::MasterWebhook;

use anyhow::Context as anyhowContext;
use anyhow::{anyhow, Result};

use rsa::pkcs8::der::zeroize::Zeroizing;
use tracing::info;

/// bot導入後，最初に実行してください
///
/// 自身のサーバのマスターwebhook，サーバ情報を登録します
/// 返信として，他のサーバが自身のサーバを拡散可能先として登録する際にコピペ可能なテキストを返します．
#[poise::command(prefix_command, track_edits, aliases("UtOwnServerData"), slash_command)]
pub async fn ut_set_own_masterhook(
    ctx: Context<'_>,
    #[description = "本サーバのサーバ名"] server_name: String,
    #[description = "本サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "署名鍵を作り直しますか？（初回はTrueにしてください）"] is_new_key: bool,
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

    let keys_pem = get_keys_pem(ctx, is_new_key).await?;

    upsert_own_server_data(
        &ctx,
        &server_name,
        &guild_id,
        &master_channel_id,
        &master_webhook_url,
        &keys_pem.private_key_pem,
        &keys_pem.public_key_pem,
    )
    .await?;

    let register_tmplate_str = format!(
        "~UtOtherServerHook {} {} {} {}",
        server_name, master_webhook_url, guild_id, keys_pem.public_key_pem
    );
    // format!("server_data: ```\n server_name: {},\n guild_id: {},\n master_channel_id: {},\n master_webhook_url: {}```", server_name, guild_id, master_channel_id, master_webhook_url)

    ctx.say(register_tmplate_str).await?;

    loged(&ctx, "サーバ情報を登録しました").await?;

    Ok(())
}

/// trueなら鍵を作り直す
/// falseなら鍵をDBから取得する
async fn get_keys_pem(ctx: Context<'_>, is_new_key: bool) -> Result<KeyPair_pem> {
    if is_new_key {
        let (private_key, public_key) = generate_keypair();
        Ok(keypair_to_pem(&private_key, &public_key))
    } else {
        let server_data = crate::db_query::own_server_data::select_own_server_data(
            &ctx.data().connection,
            ctx.guild_id()
                .ok_or(anyhow!("guildidを取得できませんでした"))?
                .0,
        )
        .await
        .context("鍵を取得できません. trueを指定してください")?;
        Ok(KeyPair_pem {
            private_key_pem: Zeroizing::new(server_data.private_key_pem),
            public_key_pem: server_data.public_key_pem,
        })
    }
}

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
pub async fn ut_set_other_masterhook(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "拡散先サーバのギルド（サーバー）ID"] guild_id: String,
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
        MasterWebhook::from(None, &server_name, guild_id, &master_webhook_url),
    )
    .await?;

    let response_msg = format!(
        "登録しました．```\nserver_name: {}, webhook_url: {}, guild_id: {}```",
        server_name, master_webhook_url, guild_id
    );
    ctx.say(&response_msg).await?;

    loged(
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

    loged(
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

    let master_webhook = master_webhook_select(connection.as_ref(), &server_name).await?;

    ctx.say(format!("master_webhook: {:?}", master_webhook))
        .await?;

    Ok(())
}
