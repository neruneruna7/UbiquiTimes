use anyhow::Context as anyhowContext;
use anyhow::Result;
use poise::serenity_prelude::{Http, Webhook};
use sqlx::SqlitePool;
use tracing::info;

use crate::db_query::member_webhooks::*;
use crate::*;

// 自動でメンバーwebhookを登録できるようにしたい
// // メンバーwebhookを登録する
//
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

// 手動でメンバーwebhookを登録する
// (prefix)UTregisterM server_name webhook_url

#[poise::command(prefix_command, track_edits, aliases("UTregisterM"), slash_command)]
pub async fn ut_member_webhook_reg_manual(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] b_server_name: String,
    // 17桁整数までしか受け取れないので，仕方なくStringにする
    #[description = "拡散先のサーバID"] b_guild_id: String,
    #[description = "拡散先のチャンネルID"] b_channel_id: String,
    #[description = "拡散先チャンネルのwebhook URL"] b_webhook_url: String,
) -> Result<()> {
    let a_member_id = ctx.author().id.0;
    let b_channel_id = b_channel_id
        .parse::<u64>()
        .context("符号なし整数を入力してください")?;
    let b_guild_id = b_guild_id
        .parse::<u64>()
        .context("符号なし整数を入力してください")?;

    // match channel_id {
    //     Ok(channel_id) => channel_id,
    //     Err(_) => {
    //         ctx.say("符号なし整数を入力してください").await?;
    //         return Ok(());
    //     }
    // };

    info!("a_member_id: {}", a_member_id);

    let connection = ctx.data().connection.clone();

    let menber_webhook = MemberWebhook::from(
        a_member_id,
        &b_server_name,
        b_guild_id,
        b_channel_id,
        &b_webhook_url,
    );

    member_webhook_upsert(connection.as_ref(), menber_webhook).await?;

    let text = "member webhook inserted";
    info!(text);

    ctx.say(text).await?;

    Ok(())
}

#[poise::command(prefix_command, track_edits, aliases("UTlist"), slash_command)]
pub async fn ut_list(ctx: Context<'_>) -> Result<()> {
    let connection = ctx.data().connection.clone();

    let member_id = ctx.author().id.0;

    let member_webhooks =
        member_webhook_select_from_member_id(connection.as_ref(), member_id).await?;

    let mut response = String::new();
    response.push_str("拡散先リスト\n --------- \n");

    for member_webhook in member_webhooks {
        response.push_str(&format!("{}\n", member_webhook.dst_server_name));
    }

    ctx.say(response).await?;

    Ok(())
}

/// メンバーwebhookを削除する
///
/// サーバー名を指定して削除します
#[poise::command(prefix_command, track_edits, aliases("UTdelete"), slash_command)]
pub async fn ut_delete(
    ctx: Context<'_>,
    #[description = "拡散先のから削除するサーバ名"] server_name: String,
) -> Result<()> {
    let connection = ctx.data().connection.clone();
    // SqliteのINTEGER型はi64になる都合で，i64に変換する
    // discordのidは18桁で構成されており，i64に収まるため変換しても問題ないと判断した
    let member_id = ctx.author().id.0;

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
pub async fn ut_times_release(
    ctx: Context<'_>,
    #[description = "拡散内容"] content: String,
) -> Result<()> {
    let username = format!("UT-{}", ctx.author().name);

    // DBからそのユーザのwebhookをすべて取得する
    let connection = ctx.data().connection.clone();

    // SqliteのINTEGER型はi64になる都合で，i64に変換する
    // discordのidは18桁で構成されており，i64に収まるため変換しても問題ないと判断した
    let member_id = ctx.author().id.0;
    let member_webhooks =
        member_webhook_select_from_member_id(connection.as_ref(), member_id).await?;

    let member_webhooks = member_webhooks
        .iter()
        .map(|m| m.dst_webhook_url.to_owned())
        .collect::<Vec<String>>();

    execute_ubiquitus(&username, &content, member_webhooks).await?;

    Ok(())
}

// 自動でメンバーwebhookを登録できるようにしたい
// // メンバーwebhookを登録する
//
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

async fn execute_ubiquitus(
    username: &str,
    content: &str,
    webhooks: Vec<String>,
) -> anyhow::Result<()> {
    // webhookを実行する
    let http = Http::new("");

    for webhook_url in webhooks.iter() {
        let webhook = Webhook::from_url(&http, webhook_url).await?;
        webhook
            .execute(&http, false, |w| w.content(content).username(username))
            .await?;
    }
    Ok(())
}
