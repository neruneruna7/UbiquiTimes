// - サーバー初期化コマンド
// - サーバー設定情報を取得する

use crate::{error::GuildIdCannotGet, global_data::Context};
use domain::{
    models::guild_data::OwnGuild,
    traits::{repositorys::OwnGuildRepository, signer_verifier::UtKeyPairGenerator},
};
use signer_verifier::key_generator::RsaKeyPair;

use domain::tracing::info;
use poise::serenity_prelude::CreateWebhook;

/// Botのセットアップを行います 導入後最初に実行してください
///
/// マネージwebhook,署名鍵を設定します
/// 専用のDiscordチャンネルを作り，そのチャンネルでこのコマンドを実行してください
/// セキュリティ上の懸念により，鍵を作りなおす場合もこのコマンドを実行してください
#[poise::command(prefix_command, track_edits, aliases("UtInitialize"), slash_command)]
// #[tracing::instrument]
pub async fn ut_initialize(
    ctx: Context<'_>,
    #[description = "本サーバのサーバ名"] server_name: String,
    #[description = "署名鍵を作り直しますか？（初回はTrueにしてください）"] _is_new_key: bool,
) -> anyhow::Result<()> {
    info!("サーバー初期化開始: {}", server_name);
    // この関数は処理に時間がかかるため，待ち時間を延ばす
    ctx.defer().await?;

    // デバッグビルドだと鍵作成の処理時間が長いため，返信が来ないことがあります
    // おそらくどこかに時間を設定するところがあるはず...
    // チャンネルからwebhookを作成
    let manage_channel_id = ctx.channel_id();
    info!("manage_channel_id: {}", manage_channel_id);

    // ビルダーを介して作るようになったようだ
    let builder = CreateWebhook::new("UbiquitimesManageWebhook");
    let manage_webhook = manage_channel_id.create_webhook(&ctx, builder).await?;
    let manage_webhook_url = manage_webhook.url()?;

    info!("manage_webhook_url: {}", manage_webhook_url);

    // guild_idを取得
    let guild_id = ctx.guild_id().ok_or(GuildIdCannotGet)?.get();

    info!("guild_id: {}", guild_id);

    // 公開鍵を作成
    let keys_generator = ctx.data().ubiquitimes_keygenerator.clone();

    // 作成し，作成した鍵をPEM形式に変換
    let keys_pem = {
        let keys = keys_generator.generate_key_pair()?;
        RsaKeyPair::to_pem(&keys)
    };

    info!("公開鍵のPEM: {}", keys_pem.public_key_pem);

    // どちらも作成に成功したらDBに保存
    let own_server = OwnGuild::new(
        guild_id,
        &server_name,
        manage_channel_id.into(),
        manage_webhook_url.as_str(),
        &keys_pem.private_key_pem,
        &keys_pem.public_key_pem,
    );
    let own_server_repository = ctx.data().own_server_repository.clone();
    own_server_repository.upsert(own_server)?;

    info!("サーバー初期化完了: {}", server_name);

    // manage_webhookのURLと公開鍵のpemを返信
    let reply = format!(
        "manage_webhookのURL: {}\n公開鍵のPEM: {}",
        manage_webhook_url, keys_pem.public_key_pem
    );

    info!("reply_message: {}", reply);
    ctx.reply(reply).await?;

    Ok(())
}

/// このサーバの情報を返します
///
/// 他のサーバが自身のサーバを拡散可能先として登録する際にコピペ可能なテキストを返します．
#[poise::command(
    prefix_command,
    track_edits,
    aliases("UtGetOwnGuildData"),
    slash_command
)]
pub async fn ut_get_own_server_data(ctx: Context<'_>) -> anyhow::Result<()> {
    // この関数は処理に時間がかかるため，待ち時間を延ばす
    ctx.defer().await?;

    // dbから取得
    let own_server_repository = ctx.data().own_server_repository.clone();
    let own_server = own_server_repository.get()?;

    // manage_webhookのURLと公開鍵のpemを返信
    let reply = format!(
        "manage_webhookのURL: {}\n公開鍵のPEM: {}",
        own_server.manage_webhook_url, own_server.public_key_pem
    );

    // 返信
    ctx.reply(reply).await?;
    Ok(())
}
