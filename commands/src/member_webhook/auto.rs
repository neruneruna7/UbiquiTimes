use crate::types::global_data::Data;
use crate::types::webhook::MemberWebhook;
use crate::{db_query::master_webhooks::master_webhook_select_all, Context, Result};

use anyhow::anyhow;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Http;

use poise::serenity_prelude::Webhook;
use serde::{Deserialize, Serialize};
use tracing::debug;
use tracing::info;

use crate::db_query::{member_webhooks, own_server_times_data::*};
use crate::db_query::{
    own_server_data::{self, *},
    own_server_times_data,
};

use crate::types::botcom::*;

/// そのサーバーでの自分のtimesであることをセットする
///
/// 本サーバにおいて，このコマンドを実行したチャンネルがあなたのTimesであるとbotに登録します．
/// 結果は実行するチャンネルに依存します．
#[poise::command(prefix_command, track_edits, aliases("UTtimesSetting"), slash_command)]
pub async fn ut_times_set(
    ctx: Context<'_>,
    #[description = "拡散時に使う名前を入力してください"] name: String,
) -> Result<()> {
    // sign_str_command(&ctx, &times, "times").await?;

    let member_id = ctx.author().id.0;
    let member_name = name;
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

    upsert_own_times_data(
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


/// あなたのTimesを拡散するための設定リクエストを送信します．
///
/// 拡散可能サーバすべてに対して，拡散設定するためのリクエストを送信します
#[poise::command(
    prefix_command,
    track_edits,
    aliases("UTtimesUbiquiSettingSend"),
    slash_command
)]
pub async fn ut_times_ubiqui_setting_send(
    ctx: Context<'_>,
    #[description = "`release`と入力してください"] release: String,
) -> Result<()> {
    sign_str_command(&ctx, &release, "release").await?;

    let connection = ctx.data().connection.clone();
    // 自身のtimesの情報を取得
    let member_times = select_own_times_data(connection.as_ref(), ctx.author().id.0).await?;

    // 自身のサーバ情報を取得
    let guild_id = ctx.guild_id().ok_or(anyhow!(""))?.0;
    let server_data = select_own_server_data(connection.as_ref(), guild_id).await?;

    // 拡散可能サーバのリストを取得
    let other_master_webhooks = master_webhook_select_all(connection.as_ref()).await?;

    let times_ubiqui_setting_send = TimesUbiquiSettingSend {
        src_member_id: member_times.member_id,
        src_master_webhook_url: server_data.master_webhook_url,
        src_channel_id: member_times.channel_id,
        src_member_webhook_url: member_times.webhook_url,
    };

    debug!(
        "times_ubiqui_setting_send: {:?}",
        &times_ubiqui_setting_send
    );

    let mut bot_com_msg = BotComMessage::from(
        &server_data.server_name,
        "次のすべての送信先サーバに送信するループ内にて，送信先のサーバ名を代入してください",
        CmdKind::TimesUbiquiSettingSend(times_ubiqui_setting_send),
    );

    info!("bot_com_msg: {:?}", &bot_com_msg);

    // ここのhttpはどうするか，空白トークンのHttpをnewするか，ctxを使うか
    for other_master_webhook in other_master_webhooks.iter() {
        let webhook = Webhook::from_url(&ctx, &other_master_webhook.webhook_url).await?;
        bot_com_msg.dst = other_master_webhook.server_name.clone();
        let serialized_msg = serde_json::to_string(&bot_com_msg)?;

        webhook
            .execute(&ctx, false, |w| w.content(serialized_msg.to_string()))
            .await?;
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
pub async fn times_ubiqui_setting_recv(
    ctx: &serenity::Context,
    data: &Data,
    src_server_name: &str,
    times_ubiqui_setting: &TimesUbiquiSettingSend,
) -> Result<()> {
    info!("拡散設定リクエストを受信しました");
    let src_member_id = times_ubiqui_setting.src_member_id;

    let connection = data.connection.clone();

    // 返送先のmasterwebhook
    let recv_master_webhook_url = times_ubiqui_setting.src_master_webhook_url.clone();
    let http = Http::new("");
    let recv_master_webhook = Webhook::from_url(&http, &recv_master_webhook_url).await?;

    // a_member_id と紐づいているtimeswebhookを取得
    let member_times_data =
        own_server_times_data::select_own_times_data(connection.as_ref(), src_member_id).await?;
    let times_webhook_url = member_times_data.webhook_url;
    let times_channel_id = member_times_data.channel_id;

    // 自身のサーバ情報を取得
    let a_server_data =
        own_server_data::select_own_server_data_without_guild_id(connection.as_ref()).await?;

    // データをTimesUbiquiSettingRecvに詰める
    let times_ubiqui_setting_recv = TimesUbiquiSettingRecv {
        src_member_id,
        dst_guild_id: a_server_data.guild_id,
        dst_channel_id: times_channel_id,
        dst_webhook_url: times_webhook_url,
    };

    // データをシリアライズ
    // ここにおいては，srcとdstがそのほかの構造体と逆になる
    // つまり，自身のサーバがsrcである
    let bot_com_msg = BotComMessage::from(
        &a_server_data.server_name,
        src_server_name,
        CmdKind::TimesUbiquiSettingRecv(times_ubiqui_setting_recv),
    );
    let serialized_msg = serde_json::to_string(&bot_com_msg)?;

    // データを送信
    recv_master_webhook
        .execute(ctx, false, |w| w.content(serialized_msg.to_string()))
        .await?;

    let my_webhook = Webhook::from_url(&http, &a_server_data.master_webhook_url).await?;
    my_webhook
        .execute(ctx, false, |w| w.content("拡散設定リクエスト 受信"))
        .await?;

    Ok(())
}

/// 拡散設定返信を受信したときの処理
pub async fn times_ubiqui_setting_set(
    _ctx: &serenity::Context,
    data: &Data,
    src_server_name: &str,
    times_ubiqui_setting: &TimesUbiquiSettingRecv,
) -> Result<()> {
    info!("拡散設定リクエストを受信しました");
    let src_member_id = times_ubiqui_setting.src_member_id;

    // 必要なデータをMemberWebhookに詰める
    let member_webhook = MemberWebhook::from(
        src_member_id,
        src_server_name,
        times_ubiqui_setting.dst_guild_id,
        times_ubiqui_setting.dst_channel_id,
        &times_ubiqui_setting.dst_webhook_url,
    );

    let connection = data.connection.clone();

    info!("times_ubiqui_setting_set: DB処理 到達");
    member_webhooks::member_webhook_upsert(connection.as_ref(), member_webhook).await?;

    Ok(())
}
