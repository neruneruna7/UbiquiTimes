use tracing::{error, info};

use crate::*;

#[poise::command(prefix_command, track_edits, aliases("UTregserver"), slash_command)]
pub async fn ut_regserver(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "拡散先サーバのギルド（サーバー）ID"] guild_id: Option<i64>,
) -> Result<()> {
    // msg.contentを分割して、server_nameとwebhook_urlを取得する
    // let mut iter = msg.content.split_whitespace();
    // let _ = iter.next().unwrap();
    // let server_name = iter.next().unwrap();
    // let guild_id = iter.next().unwrap().parse::<i64>().unwrap();
    // let webhook_url = iter.next().unwrap();

    // log
    info!(
        "server_name: {}, webhook_url: {}, guild_id: {:?}",
        server_name, master_webhook_url, guild_id
    );

    // DBに登録する
    let connection = ctx.data().connection.clone();

    master_webhook_insert(
        connection.as_ref(),
        MasterWebhook::from(None, &server_name, guild_id, &master_webhook_url),
    )
    .await?;

    Ok(())
}

#[poise::command(prefix_command, track_edits, aliases("UTgetMasterHook"), slash_command)]
pub async fn get_master_hook(
    ctx: Context<'_>,
    #[description = "webhookを確認するサーバ名"] server_name: String,
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

// 自動でメンバーwebhookを登録できるようにしたい
// // メンバーwebhookを登録する
// #[allow(non_snake_case)]
// #[poise::command(prefix_command, track_edits, slash_command)]
// async fn UTregister(
//     ctx: Context<'_>,
//     #[description = "拡散先のサーバ名"] server_name: String,
//     #[description = "拡散先のチャンネルID"] channel_id: i64,
// ) -> Result<()> {
//     // もしチャンネルにwebhookが存在していたら、それを使う
//     // なければ、新規に作成する
//     // チャンネルidから，存在しているwebhookを取得する
//     let webhooks = msg.channel_id.webhooks(&ctx).await?;

//     // UT- username という名前のwebhookがあるかどうか
//     let webhook = if let Some(webhook) = webhooks.iter().find(|w| w.name == Some(format!("UT-{}", &msg.author.name))) {
//         webhook.to_owned()
//     } else {
//         msg.channel_id.create_webhook(&ctx, format!("UT-{}", &msg.author.name)).await?
//     };

//     let my_webhook_url = webhook.url()?;

//     // さらなる記述が必要

//     Ok(())
// }

/// 手動で拡散先のサーバ，チャンネル，webhookを登録します
/// 
/// 登録する側をサーバＡ，される側をサーバＢとします．
/// サーバＢの識別名（現状，登録する側が自由に決める）
/// サーバＢの拡散先のチャンネルID
/// サーバＢの拡散先チャンネルのwebhook URL を入力してください
#[poise::command(prefix_command, track_edits, aliases("UTregisterM"), slash_command)]
pub async fn ut_member_webhook_reg_manual(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先のチャンネルID"] channel_id: i64,
    #[description = "拡散先チャンネルのwebhook URL"] webhook_url: String,
) -> Result<()> {
    let member_id = ctx.author().id.0 as i64;
    info!("member_id: {}", member_id);
    let connection = ctx.data().connection.clone();

    let menber_webhook = MemberWebhook::from(
        None,
        &server_name,
        member_id,
        channel_id,
        &webhook_url
    );

    member_webhook_insert(connection.as_ref(), menber_webhook).await?;

    let text = "member webhook inserted";
    info!(text);

    ctx.say(text).await?;

    Ok(())
}

/// 自分が拡散先に登録したサーバの一覧を表示します
#[poise::command(prefix_command, track_edits, aliases("UTlist"), slash_command)]
pub async fn ut_list(
    ctx: Context<'_>
) -> Result<()> {
    let connection = ctx.data().connection.clone();

    // SqliteのINTEGER型はi64になる都合で，i64に変換する
    // discordのidは18桁で構成されており，i64に収まるため変換しても問題ないと判断した
    let member_id = ctx.author().id.0 as i64;

    let member_webhooks = member_webhook_select_all(connection.as_ref(), member_id).await?;

    let mut response = String::new();

    for member_webhook in member_webhooks {
        response.push_str(&format!("{}\n", member_webhook.server_name));
    }

    ctx.say(response).await?;

    Ok(())
}

/// 拡散先から削除する
/// 
/// サーバー名を指定して削除します
/// 
#[poise::command(prefix_command, track_edits, aliases("UTdelete"), slash_command)]
pub async fn ut_delete(
    ctx: Context<'_>,
    #[description = "拡散先のから削除するサーバ名"] server_name: String,
) -> Result<()> {
    let connection = ctx.data().connection.clone();
    // SqliteのINTEGER型はi64になる都合で，i64に変換する
    // discordのidは18桁で構成されており，i64に収まるため変換しても問題ないと判断した
    let member_id = ctx.author().id.0 as i64;

    member_webhook_delete(connection.as_ref(), &server_name, member_id).await?;

    info!("member webhook deleted");
    ctx.say("member webhook deleted").await?;

    Ok(())
}


/// 投稿内容を拡散します. `~UT`コマンドの使用を推奨
/// 
/// contentに記述した内容を拡散します
/// このスラッシュコマンドではなく，`~UT`のプレフィックスコマンドを推奨
/// ### `~UT`の場合
/// ```
/// ~UT
/// コーラル！
/// 一度生まれたものは，そう簡単には死なないってウォルターおじが言ってた
/// ```
#[poise::command(prefix_command, track_edits, aliases("UT"), slash_command)]
pub async fn ut_execute(
    ctx: Context<'_>, 
    #[description = "拡散内容"] content: String,
) -> Result<()> {
    let username = format!("UT-{}", ctx.author().name);

    // DBからそのユーザのwebhookをすべて取得する
    let connection = ctx.data().connection.clone();

    // SqliteのINTEGER型はi64になる都合で，i64に変換する
    // discordのidは18桁で構成されており，i64に収まるため変換しても問題ないと判断した
    let member_id = ctx.author().id.0 as i64;
    let member_webhooks = member_webhook_select_all(connection.as_ref(), member_id).await?;

    let member_webhooks = member_webhooks.iter().map(|m| m.webhook_url.to_owned()).collect::<Vec<String>>();

    execute_ubiquitus(&username, &content, member_webhooks).await?;

    Ok(())
}