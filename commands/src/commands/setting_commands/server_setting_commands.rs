// - サーバー初期化コマンド
// - サーバー設定情報を取得する

use super::super::CommandsResult;
use crate::global_data::Context;
use anyhow::Context as anyhowContext;
use anyhow::Result;

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
    let manage_webhook_url = manage_channel_id
        .create_webhook(&ctx, "UbiquitimesManageWebhook")
        .await?;
    // 公開鍵を作成
    let keys_generator = 
    // どちらも作成に成功したらDBに保存

    Ok(())
}
