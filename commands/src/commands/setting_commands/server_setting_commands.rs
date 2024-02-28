// - サーバー初期化コマンド
// - サーバー設定情報を取得する

use std::ops::Deref;

use super::super::CommandsResult;
use crate::global_data::Context;
use crate::own_server::{OwnServer, OwnServerRepository};
use crate::own_server_repository::OwnServerRepository;
use crate::sign::UbiquitimesKeys;
use crate::sign::{keys_gen::RsaKeyGenerator, UbiquitimesKeyGenerator};
use anyhow::Context as anyhowContext;
use anyhow::{anyhow, Result};

/// Botのセットアップを行います 導入後最初に実行してください
///
/// マネージwebhook,署名鍵を設定します
/// 専用のDiscordチャンネルを作り，そのチャンネルでこのコマンドを実行してください
/// セキュリティ上の懸念により，鍵を作りなおす場合もこのコマンドを実行してください
#[poise::command(prefix_command, track_edits, aliases("UtInitialize"), slash_command)]
pub async fn ut_initialize(
    ctx: Context<'_>,
    #[description = "本サーバのサーバ名"] server_name: String,
    #[description = "署名鍵を作り直しますか？（初回はTrueにしてください）"] is_new_key: bool,
) -> Result<()> {
    // デバッグビルドだと鍵作成の処理時間が長いため，返信が来ないことがあります
    // おそらくどこかに時間を設定するところがあるはず...
    // チャンネルからwebhookを作成
    let manage_channel_id = ctx.channel_id();
    let manage_webhook = manage_channel_id
        .create_webhook(&ctx, "UbiquitimesManageWebhook")
        .await?;
    let manage_webhook_url = manage_webhook.url()?;

    // guild_idを取得
    let guild_id = ctx
        .guild_id()
        .ok_or(anyhow!("guild_idが取得できませんでした"))?
        .0;

    // 公開鍵を作成
    let keys_generator = ctx.data().ubiquitimes_keygenerator.clone();

    // 作成し，作成した鍵をPEM形式に変換
    let keys_pem = {
        let keys = keys_generator.generate_keys()?;
        UbiquitimesKeys::to_pem(&keys)
    };

    // どちらも作成に成功したらDBに保存
    let own_server = OwnServer::new(
        guild_id,
        &server_name,
        manage_channel_id.into(),
        manage_webhook_url.as_str(),
        &keys_pem.private_key_pem,
        &keys_pem.public_key_pem,
    );
    let own_server_repository = ctx.data().own_server_repository.clone();
    own_server_repository.upsert(own_server).await?;

    // webhookのURLと公開鍵のpemを返信
    let reply = format!(
        "manage_webhookのURL: {}\n公開鍵のPEM: {}",
        manage_webhook_url, keys_pem.public_key_pem
    );
    ctx.say(reply).await?;
    Ok(())
}
