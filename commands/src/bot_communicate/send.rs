use std::collections::{HashSet, HashMap};

use super::*;
use crate::other_server::OtherServerData;
use crate::other_server::OtherTimesData;
use crate::own_server::{OwnTimesData, OwnServerData};
use crate::{sign_str_command, loged_serenity_ctx};

use crate::global_data::Data;
use crate::loged;
use crate::{db_query::other_server_data::master_webhook_select_all, Context, Result};

use anyhow::{anyhow, Context as _};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Http;

use poise::serenity_prelude::Webhook;

use tracing::debug;
use tracing::info;

use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};


use crate::db_query::{other_server_times_data, own_server_times_data::*};
use crate::db_query::{
    own_server_data::{self, *},
    own_server_times_data,
};


use crate::sign::claims::Claims;

/// botからのメッセージを受け取ったときの処理
pub async fn bot_com_msg_recv(
    new_message: &poise::serenity_prelude::Message,
    data: &Data,
) -> Result<Option<TokenData<Claims>>> {
    // botから以外のメッセージは無視
    if !new_message.author.bot {
        return Ok(None);
    }

    // メッセージの内容をデシリアライズ. デシリアライズできない場合は無視
    let bot_com_msg: SendBotComMessage = match serde_json::from_str(&new_message.content) {
        Ok(t) => t,
        Err(_) => {
            return Ok(None);
        }
    };

    let public_key_pem_hashmap = data.public_key_pem_hashmap.read().await;

    let public_key_pem = public_key_pem_hashmap
        .get(&bot_com_msg.src_guild_id)
        .context("公開鍵が登録されていません")?;

    // info!("public_key_pem_hashmap: {:?}", &public_key_pem_hashmap);

    // let a = new_message.content

    // メッセージの内容を検証．検証できない場合は無視
    // 関係ないメッセージもこの関数を通るため，検証できなくてもそれはエラーではないと判断した
    let token = decode::<Claims>(
        &new_message.content,
        &DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).unwrap(),
        &Validation::new(Algorithm::RS256),
    )
    .ok();

    info!("token: {:?}", &token);

    Ok(token)
}

/// ut_times_ubiqui_setting_sendのために必要なデータを詰める関数
async fn get_data_for_ut_times_ubiqui_setting_send(
    ctx: &Context<'_>,
) -> Result<(OwnTimesData, OwnServerData, Vec<OtherServerData>)> {
    let connection = ctx.data().connection.clone();
    // 自身のtimesの情報を取得
    let member_times = select_own_times_data(connection.as_ref(), ctx.author().id.0).await?;

    // 自身のサーバ情報を取得
    let guild_id = ctx.guild_id().ok_or(anyhow!(""))?.0;
    let server_data = select_own_server_data(connection.as_ref(), guild_id).await?;

    // 拡散可能サーバのリストを取得
    let other_master_webhooks = master_webhook_select_all(connection.as_ref()).await?;

    Ok((member_times, server_data, other_master_webhooks))
}

/// 送信処理部分
async fn times_ubiqui_setting_send_sender(
    ctx: &Context<'_>,
    claims: &mut Claims,
    send_bot_com_msg: &mut SendBotComMessage,
    member_times: &OwnTimesData,
    server_data: &OwnServerData,
    other_master_webhooks: &Vec<OtherServerData>,
    botcom_sended: &mut HashMap<u64, HashSet<u64>>,
) -> Result<()>{
    let mut sended_guild_id = HashSet::new();
    // ここのhttpはどうするか，空白トークンのHttpをnewするか，ctxを使うか
    for other_master_webhook in other_master_webhooks.iter() {
        let webhook = Webhook::from_url(&ctx, &other_master_webhook.webhook_url).await?;
        // bot_com_msg.dst = other_master_webhook.server_name.clone();
        claims.aud = other_master_webhook.server_name.clone();

        // ここでサインする
        let token = encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(server_data.private_key_pem.as_bytes())?,
        )
        .context("署名に失敗")?;

        // 書き換えるべき値をここで書き換え
        send_bot_com_msg.dst_guild_id = other_master_webhook.guild_id;
        send_bot_com_msg.token = token;

        sended_guild_id.insert(other_master_webhook.guild_id);

        let serialized_msg = serde_json::to_string(&send_bot_com_msg)?;

        webhook
            .execute(&ctx, false, |w| w.content(serialized_msg))
            .await?;
    }

    botcom_sended.insert(member_times.member_id ,sended_guild_id);

    Ok(())
}

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

    // 必要なデータを取得
    let (member_times, server_data, other_master_webhooks) =
        get_data_for_ut_times_ubiqui_setting_send(&ctx).await?;

    // どのサーバに対して送信したかを記録する
    let mut botcom_sended = ctx.data().botcom_sended.write().await;
    *botcom_sended = HashMap::new();

    let times_ubiqui_setting_send = TimesUbiquiSettingSend {
        src_member_id: member_times.member_id,
        src_master_webhook_url: server_data.master_webhook_url.clone(),
        src_channel_id: member_times.channel_id,
        src_member_webhook_url: member_times.webhook_url.clone(),
    };

    debug!(
        "times_ubiqui_setting_send: {:?}",
        &times_ubiqui_setting_send
    );

    let mut claims = Claims::new(
        &server_data.server_name,
        server_data.guild_id,
        "ループ内で変更してください",
        CmdKind::TimesUbiquiSettingSend(times_ubiqui_setting_send),
    );

    // 送信するメッセージの内容を詰める
    // tokenはループ内で詰める
    let mut send_bot_com_msg = SendBotComMessage::new(server_data.guild_id, 0, "".to_string());

    info!("craims: {:?}", &claims);

    // claims作成
    // botcommsg作成
    // claims署名 tokenに
    // botcommsgのdst_guild__idとtokenをループ内で書き換え
    // botcommsgをシリアライズ
    // 送信

    times_ubiqui_setting_send_sender(
        &ctx,
        &mut claims,
        &mut send_bot_com_msg,
        &member_times,
        &server_data,
        &other_master_webhooks,
        &mut botcom_sended,
    ).await?;

    ctx.say("拡散設定リクエストを送信しました").await?;
    loged(&ctx, "拡散設定リクエストが送信されました").await?;

    Ok(())
}

/// ut_times_ubiqui_setting_recvのために必要なデータを詰める関数
async fn get_data_for_ut_times_ubiqui_setting_recv(
    ctx: &serenity::Context,
    data: &Data,
    times_ubiqui_setting: &TimesUbiquiSettingSend,
) -> Result<(Webhook, OwnTimesData, OwnServerData)> {
    let src_member_id = times_ubiqui_setting.src_member_id;

    let connection = data.connection.clone();

    // 返送先のOtherServerData
    let recv_master_webhook_url = times_ubiqui_setting.src_master_webhook_url.clone();
    let http = Http::new("");
    let recv_master_webhook = Webhook::from_url(&http, &recv_master_webhook_url).await?;

    // a_member_id と紐づいているtimeswebhookを取得
    let member_times_data =
        own_server_times_data::select_own_times_data(connection.as_ref(), src_member_id).await?;

    // 自身のサーバ情報を取得
    let own_server_data =
        own_server_data::select_own_server_data_without_guild_id(connection.as_ref()).await?;

    Ok((
        recv_master_webhook,
        member_times_data,
        own_server_data,
    ))
}

/// 拡散設定リクエストを受信したときの処理
pub async fn times_ubiqui_setting_recv(
    ctx: &serenity::Context,
    data: &Data,
    src_guild_id: u64,
    _src_server_name: &str,
    times_ubiqui_setting: &TimesUbiquiSettingSend,
) -> Result<()> {
    info!("拡散設定リクエストを受信しました");

    // 必要なデータを取得
    let (recv_master_webhook, member_times_data, own_server_data) =
        get_data_for_ut_times_ubiqui_setting_recv(ctx, data, times_ubiqui_setting).await?;

    let times_webhook_url = member_times_data.webhook_url;
    let times_channel_id = member_times_data.channel_id;

    // データをTimesUbiquiSettingRecvに詰める
    let times_ubiqui_setting_recv = TimesUbiquiSettingRecv {
        src_member_id: member_times_data.member_id,
        dst_guild_id: own_server_data.guild_id,
        dst_channel_id: times_channel_id,
        dst_webhook_url: times_webhook_url,
    };


    // データをシリアライズ
    // ここにおいては，srcとdstがそのほかの構造体と逆になる
    // つまり，自身のサーバがsrcである
    // ここでは署名を使わないので，tokenはNone
    let bot_com_msg = RecievedBotComMessage::new(
        own_server_data.guild_id,
        src_guild_id,
        CmdKind::TimesUbiquiSettingRecv(times_ubiqui_setting_recv),
    );

    let serialized_msg = serde_json::to_string(&bot_com_msg)?;

    // データを送信
    recv_master_webhook
        .execute(ctx, false, |w| w.content(serialized_msg.to_string()))
        .await?;

    loged_serenity_ctx(ctx, &own_server_data.master_webhook_url, "拡散設定リクエスト 受信").await?;

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

    // 必要なデータをOtherTimesDataに詰める
    let member_webhook = OtherTimesData::from(
        src_member_id,
        src_server_name,
        times_ubiqui_setting.dst_guild_id,
        times_ubiqui_setting.dst_channel_id,
        &times_ubiqui_setting.dst_webhook_url,
    );

    let connection = data.connection.clone();

    info!("times_ubiqui_setting_set: DB処理 到達");
    other_server_times_data::member_webhook_upsert(connection.as_ref(), member_webhook).await?;

    Ok(())
}
