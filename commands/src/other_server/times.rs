
use crate::own_server::OwnTimesDataTable;
use crate::*;

use anyhow::Context as anyhowContext;
use anyhow::Result;

use tracing::info;

use poise::serenity_prelude::{Http, Webhook};

use super::*;

use crate::db_query::SledTable;

/// 非推奨 手動でメンバーwebhookを登録します
#[poise::command(
    prefix_command,
    track_edits,
    aliases("UTmanualRegister"),
    slash_command
)]
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

    info!("a_member_id: {}", a_member_id);

    let other_times_data = OtherTimesData::new(
        a_member_id,
        &b_server_name,
        b_guild_id,
        b_channel_id,
        &b_webhook_url,
    );

    let db = ctx.data().connection.clone();

    let other_times_table = OtherTimesDataTable::new(db.as_ref());

    other_times_table.upsert(
        &other_times_data.dst_guild_id.to_string(),
        &other_times_data,
    );

    let text = "member webhook inserted";
    info!(text);

    ctx.say(text).await?;

    Ok(())
}

/// あなたのメンバー拡散先リストを表示します
///
/// あなたのメンバーウェブフックを登録しているサーバー名を，一覧表示します
#[poise::command(prefix_command, track_edits, aliases("UTlist"), slash_command)]
pub async fn ut_list(ctx: Context<'_>) -> Result<()> {
    let member_id = ctx.author().id.0;

    let other_times_data_vec = {
        let db = ctx.data().connection.clone();
        let other_times_table = OtherTimesDataTable::new(db.as_ref());
        other_times_table.read_all()?
    };

    // 自身のmember_idと一致するものを抽出する
    let member_webhooks = other_times_data_vec
        .into_iter()
        .filter(|m| m.src_member_id == member_id)
        .collect::<Vec<OtherTimesData>>();

    let mut response = String::new();
    response.push_str("拡散先リスト\n --------- \n```");

    for member_webhook in member_webhooks {
        response.push_str(&format!("{}\n", member_webhook.dst_server_name));
    }
    response.push_str("```");

    ctx.say(response).await?;

    Ok(())
}

/// メンバー拡散先を削除する
///
/// サーバー名を指定してメンバーウェブフックを削除します
#[poise::command(prefix_command, track_edits, aliases("UTdelete"), slash_command)]
pub async fn ut_delete(
    ctx: Context<'_>,
    #[description = "拡散先のから削除するサーバ名"] server_name: String,
) -> Result<()> {
    let _member_id = ctx.author().id.0;

    let db = ctx.data().connection.clone();

    let other_times_table = OtherTimesDataTable::new(db.as_ref());
    // サーバ名をもとにguild_idを取得
    let guild_id = {
        let other_times_data = other_times_table
            .read(&server_name)
            .context("other_times_dataの読み込みに失敗しました")?
            .context("other_times_dataが存在しません")?;
        other_times_data.dst_guild_id
    };

    other_times_table.delete(&guild_id.to_string())?;

    info!("member webhook deleted");
    ctx.say("member webhook deleted").await?;

    Ok(())
}

/// 投稿内容を拡散します. `~UT`コマンドの使用を推奨
///
/// contentに記述した内容を拡散します
/// このスラッシュコマンドではなく，`~UT`のプレフィックスコマンドを推奨
/// スラッシュコマンドの場合，拡散元のサーバでは内容が表示されません
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
    let _username = format!("UT-{}", ctx.author().name);

    let db = ctx.data().connection.clone();
    let times_data = {
        let own_server_times_table = OwnTimesDataTable::new(db.as_ref());
        own_server_times_table
            .read(&ctx.author().id.0.to_string())
            .context("own_server_times_dataの読み込みに失敗しました")?
            .context("own_server_times_dataが存在しません")?
    };

    // webhookのusernameを設定する
    let username = format!("UT-{}", times_data.member_name);

    // DBからそのユーザのwebhookをすべて取得する

    let _member_id = ctx.author().id.0;

    let other_times_data_vec = {
        let other_times_table = OtherTimesDataTable::new(db.as_ref());
        other_times_table.read_all()?
    };

    let member_webhooks = other_times_data_vec
        .iter()
        .map(|m| m.dst_webhook_url.to_owned())
        .collect::<Vec<String>>();

    execute_ubiquitus(&ctx, &username, &content, member_webhooks).await?;

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
    ctx: &Context<'_>,
    username: &str,
    content: &str,
    webhooks: Vec<String>,
) -> anyhow::Result<()> {
    // avatar_urlを取得する
    let avatar_url = ctx.author().avatar_url().unwrap_or_default();

    // webhookを実行する
    let http = Http::new("");

    for webhook_url in webhooks.iter() {
        let webhook = Webhook::from_url(&http, webhook_url).await?;
        webhook
            .execute(&http, false, |w| {
                w.content(content)
                    .username(username)
                    .avatar_url(&avatar_url)
            })
            .await?;
    }
    Ok(())
}
