use crate::Data;
use crate::{Context, Result, SqlitePool, db_query::master_webhooks::master_webhook_select_all};

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Member;
use poise::serenity_prelude::Webhook;
use poise::serenity_prelude::Http;
use tracing::debug;
use tracing::info;
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use crate::db_query::{a_server_data::*, a_member_times_data};
use crate::db_query::a_member_times_data::*;




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
pub struct BotComMessage {
    pub src: String,
    pub dst: String,
    pub cmd: CmdKind,
    pub ttl: usize,
}

impl BotComMessage {
    fn from(
        src: &str,
        dst: &str,
        cmd: CmdKind,
    ) -> BotComMessage {
        let src = src.to_string();
        let dst = dst.to_string();
        let ttl = 4;
        Self { src, dst, cmd, ttl }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CmdKind {
    TimesUbiquiSettingSend(TimesUbiquiSetting),
    TimesUbiquiSettingRecv(TimesUbiquiSettingRecv),
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimesUbiquiSetting {
    member_id: u64,
    master_webhook_url: String,
    channel_id: u64,
    webhook_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimesUbiquiSettingRecv {
    pub a_member_id: u64,
    pub b_guild_id: u64,
    pub b_channel_id: u64,
    pub b_webhook_url: String,
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
    let member_times = select_member_times(connection.as_ref(), ctx.author().id.0).await?;

    // 自身のサーバ情報を取得
    let guild_id = ctx.guild_id().ok_or(anyhow!(""))?.0;
    let server_data = select_a_server_data(connection.as_ref(), guild_id).await?;

    // 拡散可能サーバのリストを取得
    let other_master_webhooks = master_webhook_select_all(connection.as_ref()).await?;

    let times_ubiqui_setting_send = TimesUbiquiSetting {
        member_id: member_times.member_id,
        master_webhook_url: server_data.master_webhook_url,
        channel_id: member_times.channel_id,
        webhook_url: member_times.webhook_url,
    };

    debug!("times_ubiqui_setting_send: {:?}", &times_ubiqui_setting_send);

    let bot_com_msg = BotComMessage::from(
        &ctx.author().name,
        &server_data.server_name,
        CmdKind::TimesUbiquiSettingSend(times_ubiqui_setting_send),
    );
    let serialized_msg = serde_json::to_string(&bot_com_msg)?;

    info!("serialized_msg: {}", &serialized_msg);


    // ここのhttpはどうするか，空白トークンのHttpをnewするか，ctxを使うか
    for other_master_webhook in other_master_webhooks.iter(){
        let webhook = Webhook::from_url(&ctx, &other_master_webhook.webhook_url).await?;



        webhook.execute(&ctx, false, |w| {
            w.content(format!("{}: {}", "TimesUbiquiSetting", &serialized_msg))
        }).await?;
    } 

    ctx.say("拡散設定リクエストを送信しました").await?;

    Ok(())
}

/// botからのメッセージを受け取ったときの処理
pub async fn bot_com_msg_recv(
    new_message: &poise::serenity_prelude::Message,
) -> Option<BotComMessage> {
    // botから以外のメッセージは無視
    if !new_message.author.bot {
        return None;
    }

    // メッセージの内容をデシリアライズ. デシリアライズできない場合は無視
    let bot_com_msg: BotComMessage = match serde_json::from_str(&new_message.content) {
        Ok(t) => t,
        Err(_) => {
            return None;
        }
    };

    info!("bot_com_msg: {:?}", &bot_com_msg);

    Some(bot_com_msg)
}

/// 拡散設定リクエストを受信したときの処理
async fn times_ubiqui_setting_recv(
    ctx: &serenity::Context,
    data: &Data,
    times_ubiqui_setting: &TimesUbiquiSetting,
) -> Result<()> {
    let a_member_id = times_ubiqui_setting.member_id;
    
    let connection = data.connection.clone();

    // 返送先のmasterwebhook
    let recv_master_webhook_url = times_ubiqui_setting.master_webhook_url.clone();
    let http = Http::new("");
    let recv_master_webhook = Webhook::from_url(http, &recv_master_webhook_url).await?;

    // a_member_id と紐づいているtimeswebhookを取得
    let member_times_data = a_member_times_data::select_member_times(connection.as_ref(), a_member_id).await?;
    let times_webhook_url = member_times_data.webhook_url;
    let times_channel_id = member_times_data.channel_id;


    // データをTimesUbiquiSettingRecvに詰める
    // let times_ubiqui_setting_recv = TimesUbiquiSettingRecv {
    //     a_member_id,
    //     b_guild_id: ctx.guild_id().ok_or(anyhow!(""))?.0,
    //     b_channel_id: times_ubiqui_setting.channel_id,
    //     b_webhook_url: times_webhook_url,
    // };

    Ok(())
}