// - 記述内容を各サーバに拡散するコマンド

use crate::other_server_repository::OtherTimesRepository;
use crate::{global_data::Context, own_server_repository::OwnTimesRepository};
use anyhow::Context as anyhowContext;
use anyhow::Result;
use poise::serenity_prelude::{ExecuteWebhook, Http, Webhook};

/// 投稿内容を拡散します. `~UT`コマンドの使用を推奨
///
/// contentに記述した内容を拡散します
/// このスラッシュコマンドではなく，`~UT`のプレフィックスコマンドを推奨
/// スラッシュコマンドの場合，拡散元のサーバでは内容が表示されません
/// ### `~UT`の場合
/// 
/// ~UT
/// 一度生まれたものは，そう簡単には死なない
/// 
#[poise::command(prefix_command, track_edits, aliases("UT"), slash_command)]
pub async fn ut_times_release(
    ctx: Context<'_>,
    #[description = "拡散内容"] content: String,
) -> Result<()> {
    let _username = format!("UT-{}", ctx.author().name);
    let member_id = ctx.author().id.get();

    // let db = ctx.data().connection.clone();
    // let times_data = OwnTimesData::db_read(db.as_ref(), member_id)
    //     .context("own_server_times_dataの読み込みに失敗しました")?
    //     .context("own_server_times_dataが存在しません")?;
    let own_times_repository = ctx.data().own_times_repository.clone();
    let times_data = own_times_repository
        .get(member_id)
        .await
        .context("own_server_times_dataの読み込みに失敗しました")?
        .context("own_server_times_dataが存在しません")?;

    // webhookのusernameを設定する
    let username = format!("UT-{}", times_data.member_name);

    // DBからそのユーザのwebhookをすべて取得する
    // let _member_id = ctx.author().id.0;

    // let other_times_data_vec = OtherTimesData::db_read_all(db.as_ref())?;
    let other_times_repository = ctx.data().other_times_repository.clone();

    let other_times_data_vec = other_times_repository
        .get_all()
        .await
        .context("other_server_times_dataの読み込みに失敗しました")?;
    let member_webhooks = other_times_data_vec
        .iter()
        .map(|m| m.dst_webhook_url.to_owned())
        .collect::<Vec<String>>();

    execute_ubiquitus(&ctx, &username, &content, member_webhooks).await?;

    Ok(())
}

async fn execute_ubiquitus(
    ctx: &Context<'_>,
    username: &str,
    content: &str,
    webhooks: Vec<String>,
) -> anyhow::Result<()> {
    // avatar_urlを取得する
    // 要はアイコンの画像
    let avatar_url = ctx.author().avatar_url().unwrap_or_default();

    // webhookを実行する
    let http = Http::new("");

    for webhook_url in webhooks.iter() {
        let webhook = Webhook::from_url(&http, webhook_url).await?;
        let builder = ExecuteWebhook::new()
            .content(content)
            .username(username)
            .avatar_url(&avatar_url);
        webhook.execute(&http, false, builder).await?;
    }
    Ok(())
}

/// Responds with "world!"
#[poise::command(slash_command, prefix_command)]
pub async fn hello(ctx: Context<'_>) -> anyhow::Result<()> {
    // 動作確認用
    ctx.say("world!").await?;
    Ok(())
}
