use crate::{Context, Result, SqlitePool, db_query::master_webhooks::master_webhook_select_all};

use tracing::info;
use serde::{Deserialize, Serialize};

use crate::db_query::a_member_times_data::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct BotComMessage {
    pub src: String,
    pub dst: String,
    pub cmd: CmdKind,
    pub ttl: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CmdKind {
    TimesUbiquiSetting(TimesUbiquiSetting),
}

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





// #[poise::command(prefix_command, track_edits, aliases("UTtimesUnset"), slash_command)]
// pub async fn ut_times_unset(
//     ctx: Context<'_>,
//     #[description = "`untimes`と入力してください"] untimes: String,

#[derive(Debug, Serialize, Deserialize)]
pub struct TimesUbiquiSetting {
    member_id: u64,
    master_webhook: String,
    channel_id: u64,
    servername: String,
    webhook_url: String,
}

/// あなたのTimesを拡散するための設定リクエストを送信します．
/// 
/// 拡散可能サーバすべてに対して，拡散設定するためのリクエストを送信します

#[poise::command(prefix_command, track_edits, aliases("UTtimesUbiquiSettingSend"), slash_command)]
pub async fn ut_times_ubiqui_setting_send(
    ctx: Context<'_>,
    #[description = "`release`と入力してください"] release: String,
) -> Result<()> {
    sign_str_command(&ctx, &release, "release").await?;

    let connection = ctx.data().connection.clone();
    // 自身のtimesの情報を取得
    

    // 拡散可能サーバのリストを取得
    let master_webhooks = master_webhook_select_all(connection.as_ref()).await?;



    Ok(())
}