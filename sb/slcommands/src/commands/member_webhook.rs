use poise::serenity_prelude::{Http, Webhook};
use sqlx::SqlitePool;
use anyhow::Result;
use tracing::info;

use crate::Context;

#[derive(Debug)]
// 個々人が持つwebhook
pub struct MemberWebhook {
    pub id: Option<i64>,
    pub server_name: String,
    pub member_id: u64,
    // sqliteがi64しか扱えないため，しかたなくStringを使う
    pub channel_id: u64,
    pub webhook_url: String,
}

impl MemberWebhook {
    fn from(
        _id: Option<i64>,
        server_name: &str,
        member_id: u64,
        channel_id: u64,
        webhook_url: &str,
    ) -> Self {
        Self {
            id: None,
            server_name: server_name.to_string(),
            member_id,
            channel_id,
            webhook_url: webhook_url.to_string(),
        }
    }

    fn from_row(
        _id: Option<i64>,
        server_name: &str,
        member_id: &str,
        channel_id: &str,
        webhook_url: &str,
    ) -> Result<Self> {
        Ok(

        Self {
            id: None,
            server_name: server_name.to_string(),
            member_id: member_id.parse()?,
            channel_id: channel_id.parse()?,
            webhook_url: webhook_url.to_string(),
        }
    )
    }
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

// 手動でメンバーwebhookを登録する
// (prefix)UTregisterM server_name webhook_url

#[poise::command(prefix_command, track_edits, aliases("UTregisterM"), slash_command)]
pub async fn ut_member_webhook_reg_manual(
    ctx: Context<'_>,
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先のチャンネルID"] channel_id: u64,
    #[description = "拡散先チャンネルのwebhook URL"] webhook_url: String,
) -> Result<()> {
    let member_id = ctx.author().id.0;
    info!("member_id: {}", member_id);

    let connection = ctx.data().connection.clone();

    let menber_webhook =
        MemberWebhook::from(None, &server_name, member_id, channel_id, &webhook_url);

    member_webhook_insert(connection.as_ref(), menber_webhook).await?;

    let text = "member webhook inserted";
    info!(text);

    ctx.say(text).await?;

    Ok(())
}

#[poise::command(prefix_command, track_edits, aliases("UTlist"), slash_command)]
pub async fn ut_list(ctx: Context<'_>) -> Result<()> {
    let connection = ctx.data().connection.clone();

    let member_id = ctx.author().id.0;

    let member_webhooks = member_webhook_select_from_member_id(connection.as_ref(), member_id).await?;

    let mut response = String::new();

    for member_webhook in member_webhooks {
        response.push_str(&format!("{}\n", member_webhook.server_name));
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
    let member_webhooks = member_webhook_select_from_member_id(connection.as_ref(), member_id).await?;

    let member_webhooks = member_webhooks
        .iter()
        .map(|m| m.webhook_url.to_owned())
        .collect::<Vec<String>>();

    execute_ubiquitus(&username, &content, member_webhooks).await?;

    Ok(())
}

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


// メンバーwebhookの登録
async fn member_webhook_insert(
    connection: &SqlitePool,
    member_webhook: MemberWebhook,
) -> anyhow::Result<()> {
    let member_id = member_webhook.member_id.to_string();
    let channel_id = member_webhook.channel_id.to_string();

    sqlx::query!(
        r#"
        INSERT INTO member_webhooks (server_name, member_id, channel_id, webhook_url)
        VALUES(?, ?, ?, ?);
        "#,
        member_webhook.server_name,
        member_id,
        channel_id,
        member_webhook.webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}

// メンバーwebhookの取得
pub async fn member_webhook_select(
    connection: &SqlitePool,
    server_name: &str,
    member_id: u64,
) -> Result<MemberWebhook> {
    let member_id = member_id.to_string();
    let row = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks WHERE server_name = ? AND member_id = ?;
        "#,
        server_name,
        member_id,
    )
    .fetch_one(connection)
    .await?;

    let member_webhook = MemberWebhook::from_row(
        Some(row.id),
        &row.server_name,
        &row.member_id,
        &row.channel_id,
        &row.webhook_url,
    )?;

    Ok(member_webhook)
}

// メンバーidと一致するメンバーwebhookの全取得
// 
pub async fn member_webhook_select_from_member_id(
    connection: &SqlitePool,
    member_id: u64,
) -> Result<Vec<MemberWebhook>> {
    let member_id = member_id.to_string();
    let rows = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks WHERE member_id = ?;
        "#,
        member_id,
    )
    .fetch_all(connection)
    .await?;

    let mut member_webhook_list = Vec::new();
    for row in rows {
        let member_webhook = MemberWebhook::from_row(
            Some(row.id),
            &row.server_name,
            &row.member_id,
            &row.channel_id,
            &row.webhook_url,
        )?;
        member_webhook_list.push(member_webhook);
    }

    Ok(member_webhook_list)
}


pub async fn member_webhook_select_all(
    connection: &SqlitePool,
) -> Result<Vec<MemberWebhook>> {
    let rows = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks;
        "#
    )
    .fetch_all(connection)
    .await?;

    let mut member_webhook_list = Vec::new();
    for row in rows {
        let member_webhook = MemberWebhook::from_row(
            Some(row.id),
            &row.server_name,
            &row.member_id,
            &row.channel_id,
            &row.webhook_url,
        )?;
        member_webhook_list.push(member_webhook);
    }

    Ok(member_webhook_list)
}

// servername, member_idを指定してメンバーwebhookを削除する
pub async fn member_webhook_delete(
    connection: &SqlitePool,
    server_name: &str,
    member_id: u64,
) -> Result<()> {
    let member_id = member_id.to_string();
    sqlx::query!(
        r#"
        DELETE FROM member_webhooks WHERE server_name = ? AND member_id = ?;
        "#,
        server_name,
        member_id
    )
    .execute(connection)
    .await?;

    Ok(())
}