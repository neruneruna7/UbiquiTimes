use crate::*;

use std::sync::Arc;

use anyhow::{anyhow, Result};

use poise::serenity_prelude as serenity;

use serenity::{
    async_trait,
    http::Http,
    model::{channel::Message, gateway::Ready},
    webhook::Webhook,
};

use sqlx::SqlitePool;
use tracing::error;

// Types used by all command functions
// すべてのコマンド関数で使用される型
// type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

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

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
struct MasterWebhook {
    id: Option<i64>,
    server_name: String,
    guild_id: Option<i64>,
    webhook_url: String,
}

impl MasterWebhook {
    fn from(id: Option<i64>, server_name: &str, guild_id: Option<i64>, webhook_url: &str) -> Self {
        Self {
            id: None,
            server_name: server_name.to_string(),
            guild_id,
            webhook_url: webhook_url.to_string(),
        }
    }
}

#[derive(Debug)]
// 個々人が持つwebhook
struct MemberWebhook {
    id: Option<i64>,
    server_name: String,
    member_id: i64,
    channel_id: i64,
    webhook_url: String,
}

impl MemberWebhook {
    fn from(
        id: Option<i64>,
        server_name: &str,
        member_id: i64,
        channel_id: i64,
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
}

// // ContextからDbを取得する
// // poiseにより簡単に取得できるようになったので不要の可能性が高い
// // 一旦コメントアウト
// async fn get_db(ctx: Context<'_>) -> Option<Arc<SqlitePool>> {
//     let data_read = ctx.data.read().await;
//     // let data_read = ctx.data().connection.clone();
//     let db = data_read.get::<UtDb>();

//     match db {
//         Some(db) => {
//             let db = db.clone();
//             Some(db)
//         }
//         None => {
//             error!("db is None");
//             None
//         }
//     }
// }

async fn master_webhook_insert(
    connection: &SqlitePool,
    server_webhook: MasterWebhook,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO master_webhooks (server_name, guild_id, webhook_url)
        VALUES(?, ?, ?);
        "#,
        server_webhook.server_name,
        server_webhook.guild_id,
        server_webhook.webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}

async fn master_webhook_select(
    connection: &SqlitePool,
    server_name: &str,
) -> anyhow::Result<MasterWebhook> {
    let row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks WHERE server_name = ?;
        "#,
        server_name
    )
    .fetch_one(connection)
    .await?;

    let master_webhook = MasterWebhook::from(
        Some(row.id),
        &row.server_name,
        Some(row.guild_id),
        &row.webhook_url,
    );

    Ok(master_webhook)
}

// すべてのマスターwebhookを取得する
// 複数の行がとれるので、Vecに格納して返す
async fn master_webhook_select_all(
    connection: &SqlitePool,
    _server_name: &str,
) -> anyhow::Result<()> {
    let _row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks;
        "#,
    )
    .fetch_one(connection)
    .await?;

    // let master_webhook = MasterWebhook {
    //     id: Some(row.id),
    //     server_name: row.server_name,
    //     webhook_url: row.webhook_url,
    // };

    // Ok(master_webhook)

    Ok(())
}

// メンバーwebhookの登録
async fn member_webhook_insert(
    connection: &SqlitePool,
    member_webhook: MemberWebhook,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO member_webhooks (server_name, member_id, channel_id, webhook_url)
        VALUES(?, ?, ?, ?);
        "#,
        member_webhook.server_name,
        member_webhook.member_id,
        member_webhook.channel_id,
        member_webhook.webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}

// メンバーwebhookの取得
async fn member_webhook_select(
    connection: &SqlitePool,
    server_name: &str,
    member_id: i64,
) -> anyhow::Result<MemberWebhook> {
    let row = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks WHERE server_name = ? AND member_id = ?;
        "#,
        server_name,
        member_id
    )
    .fetch_one(connection)
    .await?;

    let member_webhook = MemberWebhook::from(
        Some(row.id),
        &row.server_name,
        row.member_id,
        row.channel_id,
        &row.webhook_url,
    );

    Ok(member_webhook)
}


// メンバーwebhookの全取得
async fn member_webhook_select_all(
    connection: &SqlitePool,
    // server_name: &str,
    member_id: i64,
) -> anyhow::Result<Vec<MemberWebhook>> {
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
        let member_webhook = MemberWebhook::from(
            Some(row.id), 
            &row.server_name, 
            row.member_id, 
            row.channel_id,
            &row.webhook_url);
        member_webhook_list.push(member_webhook);
    }

    Ok(member_webhook_list)
}

// servername, member_idを指定してメンバーwebhookを削除する
async fn member_webhook_delete(
    connection: &SqlitePool,
    server_name: &str,
    member_id: i64,
) -> anyhow::Result<()> {
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

async fn create_webhook_from_channel(
    ctx: Context<'_>,
    msg: &Message,
    name: &str,
) -> anyhow::Result<Webhook> {
    let webhook = msg.channel_id.create_webhook(ctx, name).await?;
    Ok(webhook)
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<()> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

// /// Vote for something
// ///
// /// Enter `~vote pumpkin` to vote for pumpkins
// #[poise::command(prefix_command, slash_command)]
// pub async fn vote(
//     ctx: Context<'_>,
//     #[description = "What to vote for"] choice: String,
// ) -> Result<()> {
//     // Lock the Mutex in a block {} so the Mutex isn't locked across an await point
//     let num_votes = {
//         let mut hash_map = ctx.data().votes.lock().unwrap();
//         let num_votes = hash_map.entry(choice.clone()).or_default();
//         *num_votes += 1;
//         *num_votes
//     };

//     let response = format!("Successfully voted for {choice}. {choice} now has {num_votes} votes!");
//     ctx.say(response).await?;
//     Ok(())
// }

// /// Retrieve number of votes
// ///
// /// Retrieve the number of votes either in general, or for a specific choice:
// /// ```
// /// ~getvotes
// /// ~getvotes pumpkin
// /// ```
// #[poise::command(prefix_command, track_edits, aliases("votes"), slash_command)]
// pub async fn getvotes(
//     ctx: Context<'_>,
//     #[description = "Choice to retrieve votes for"] choice: Option<String>,
// ) -> Result<()> {
//     if let Some(choice) = choice {
//         let num_votes = *ctx.data().votes.lock().unwrap().get(&choice).unwrap_or(&0);
//         let response = match num_votes {
//             0 => format!("Nobody has voted for {} yet", choice),
//             _ => format!("{} people have voted for {}", num_votes, choice),
//         };
//         ctx.say(response).await?;
//     } else {
//         let mut response = String::new();
//         for (choice, num_votes) in ctx.data().votes.lock().unwrap().iter() {
//             response += &format!("{}: {} votes", choice, num_votes);
//         }

//         if response.is_empty() {
//             response += "Nobody has voted for anything yet :(";
//         }

//         ctx.say(response).await?;
//     };

//     Ok(())
// }

/// 拡散可能なサーバ一覧を表示する

#[poise::command(prefix_command, track_edits, aliases("UTserverlist"), slash_command)]
pub async fn ut_serverlist(ctx: Context<'_>) -> Result<()> {
    // DBから取得する
    let connection = ctx.data().connection.clone();

    let row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks;
        "#
    )
    .fetch_all(connection.as_ref())
    .await?;

    let mut response = String::new();
    for row in row {
        response.push_str(&format!("{}\n", row.server_name));
    }

    ctx.say(response).await?;
    Ok(())
}

use tracing::{info};

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