use crate::{Context, Data, Error, Result, SqlitePool};

use poise::serenity_prelude::{channel, guild, Http};
use serde::{Deserialize, Serialize};
use sqlx::Sqlite;
use tracing::info;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct BotComMessage {
//     pub src: String,
//     pub dst: String,
//     pub cmd: CmdKind,
//     pub ttl: usize,
// }

/// そのサーバーでの自分のtimesであることをセットする
///
/// 本サーバにおいて，このコマンドを実行したチャンネルがあなたのTimesであるとbotに登録します．
/// 結果は実行するチャンネルに依存します．
#[poise::command(prefix_command, track_edits, aliases("UTtimesSetting"), slash_command)]
pub async fn ut_times_set(
    ctx: Context<'_>,
    #[description = "`times`と入力してください"] times: String,
) -> Result<()> {
    sign_str_command(&ctx, &times, "times").await?;

    let member_id = ctx.author().id.0;
    let member_name = ctx.author().name.clone();
    let channel_id = ctx.channel_id().0;

    let webhook_name = format!("UT-{}", member_id);
    let webhook = ctx.channel_id().create_webhook(&ctx, webhook_name).await;

    info!("{:?}", webhook);

    let webhook_url = match webhook {
        Ok(t) => t.url()?,
        Err(e) => {
            let m = format!("webhookの作成に失敗しました: {}", e);
            ctx.say(&m).await?;
            return Err(anyhow::anyhow!(m));
        }
    };

    let connection = ctx.data().connection.clone();

    upsert_member_times(
        connection.as_ref(),
        member_id,
        &member_name,
        channel_id,
        &webhook_url,
    )
    .await?;

    ctx.say("このチャンネルを，本サーバでのあなたのTimesとして登録しました")
        .await?;

    Ok(())
}

/// 自身のtimesを解除する
///
/// 本サーバにおいて，あなたの登録されているTimesを削除します.
/// 結果は実行するチャンネルに依存しません．
/// どのチャンネルから実行しても同じ内容が実行されます．
#[poise::command(prefix_command, track_edits, aliases("UTtimesUnset"), slash_command)]
pub async fn ut_times_unset(
    ctx: Context<'_>,
    #[description = "`untimes`と入力してください"] untimes: String,
) -> Result<()> {
    sign_str_command(&ctx, &untimes, "untimes").await?;

    let member_id = ctx.author().id.0.to_string();
    let connection = ctx.data().connection.clone();

    let _a = sqlx::query!(
        r#"
        DELETE FROM a_member_times_data
        WHERE member_id = ?
        "#,
        member_id,
    )
    .execute(connection.as_ref())
    .await?;

    ctx.say("本サーバでのあなたのTimes登録を削除しました")
        .await?;

    Ok(())
}

/// デバッグ用に member_times_data を全て表示する
#[poise::command(prefix_command, track_edits, aliases("UTtimesShow"), slash_command)]
pub async fn ut_times_show(ctx: Context<'_>) -> Result<()> {
    let connection = ctx.data().connection.clone();

    let member_times = sqlx::query!(
        r#"
        SELECT * FROM a_member_times_data
        "#,
    )
    .fetch_all(connection.as_ref())
    .await?;

    let mut response = String::new();
    for member_time in member_times {
        response.push_str(&format!(
            "{}: times_channel_id: {}\n",
            member_time.member_name, member_time.member_id
        ));
    }

    ctx.say(response).await?;
    Ok(())
}

async fn sign_str_command(ctx: &Context<'_>, enter_str: &str, sign_str: &str) -> Result<()> {
    let err_text = format!("{}と入力してください", sign_str);
    if enter_str != sign_str {
        ctx.say(&err_text).await?;
        return Err(anyhow::anyhow!(err_text));
    }

    Ok(())
}

// sqliteにmember自身のtimes情報をupsertする
async fn upsert_member_times(
    connection: &SqlitePool,
    member_id: u64,
    member_name: &str,
    channel_id: u64,
    webhook_url: &str,
) -> Result<()> {
    // let mut conn = pool.acquire().await?;
    let member_id = member_id.to_string();
    let channel_id = channel_id.to_string();
    let _a = sqlx::query!(
        r#"
        INSERT INTO a_member_times_data (member_id, member_name, channel_id, webhook_url)
        VALUES (?, ?, ?, ?)
        ON CONFLICT (member_id) DO UPDATE SET member_name = ?, channel_id = ?, webhook_url = ?
        "#,
        member_id,
        member_name,
        channel_id,
        webhook_url,
        member_name,
        channel_id,
        webhook_url,
    )
    .execute(connection)
    .await?;

    info!("{:?}", _a);

    Ok(())
}
