use std::collections::{HashMap, HashSet};

use super::*;

use crate::other_server::OtherServerData;
use crate::other_server::OtherTimesData;
use crate::own_server::{OwnServerData, OwnTimesData};
use crate::{logged_serenity_ctx, sign_str_command};

use crate::global_data::Data;
use crate::logged;
use crate::{Context, Result};

use anyhow::{anyhow, Context as _};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Http;

use poise::serenity_prelude::Webhook;

use tracing::debug;
use tracing::info;

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::sign::claims::Claims;

/// botからのメッセージを受け取ったときの処理
pub async fn bot_com_msg_recv(
    new_message: &poise::serenity_prelude::Message,
    _data: &Data,
) -> Option<BotComMessage> {
    //Result<Option<TokenData<Claims>>>
    // botから以外のメッセージは無視
    if !new_message.author.bot {
        return None;
    }
    serde_json::from_str(&new_message.content).ok()?
}

/// ut_times_ubiqui_setting_sendのために必要なデータを詰める関数
async fn get_data_for_ut_times_ubiqui_setting_send(
    ctx: &Context<'_>,
) -> Result<(OwnTimesData, OwnServerData, Vec<OtherServerData>)> {
    let member_id = ctx.author().id.0;
    let db = ctx.data().connection.clone();
    // 自身のtimesの情報を取得
    let own_times_data = OwnTimesData::db_read(db.as_ref(), member_id)?
        .context("own_times_dataが登録されていません")?;

    // 自身のサーバ情報を取得
    let _guild_id = ctx.guild_id().ok_or(anyhow!(""))?.0;
    // let server_data = select_own_server_data(connection.as_ref(), guild_id).await?;
    let own_server_data =
        OwnServerData::db_read(db.as_ref())?.context("own_server_dataが登録されていません")?;

    // 拡散可能サーバのリストを取得

    let other_server_data = OtherServerData::db_read_all(db.as_ref())?;

    Ok((own_times_data, own_server_data, other_server_data))
}

/// 送信処理部分
async fn times_ubiqui_setting_send_sender(
    ctx: &Context<'_>,
    claims: &mut Claims,
    send_bot_com_msg: &mut BotComMessage,
    member_times: &OwnTimesData,
    server_data: &OwnServerData,
    other_master_webhooks: &Vec<OtherServerData>,
    botcom_sended: &mut HashMap<u64, HashSet<u64>>,
) -> Result<()> {
    let mut sended_guild_id = HashSet::new();
    // ここのhttpはどうするか，空白トークンのHttpをnewするか，ctxを使うか
    for other_master_webhook in other_master_webhooks.iter() {
        info!(
            "url is :{} {}",
            &other_master_webhook.webhook_url, &other_master_webhook.server_name
        );

        let http = Http::new("");
        let webhook = Webhook::from_url(http, &other_master_webhook.webhook_url).await?;
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
        send_bot_com_msg.cmd_kind = CmdKind::TimesUbiquiSettingSendToken(token);

        sended_guild_id.insert(other_master_webhook.guild_id);

        let serialized_msg = serde_json::to_string(&send_bot_com_msg)?;

        webhook
            .execute(&ctx, false, |w| w.content(serialized_msg))
            .await?;
    }

    botcom_sended.insert(member_times.member_id, sended_guild_id);

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
        "このフィールドはループ内で変更してください",
        times_ubiqui_setting_send,
    );

    // 送信するメッセージの内容を詰める
    // CmdKindはループ内で詰める
    let mut send_bot_com_msg = BotComMessage::new(server_data.guild_id, 0, CmdKind::None);

    info!("craims: {:?}", &claims);

    // claims作成
    // botcommsg作成
    // claims署名 tokenに
    // botcommsgのdst_guild__idとtokenをループ内で書き換え
    // botcommsgをシリアライズ
    // 送信

    // どのサーバに対して送信したかを記録する
    let mut botcom_sended = ctx.data().botcom_sended.write().await;
    // メンバーに紐づく送信記録をリセット
    botcom_sended.insert(member_times.member_id, HashSet::new());

    times_ubiqui_setting_send_sender(
        &ctx,
        &mut claims,
        &mut send_bot_com_msg,
        &member_times,
        &server_data,
        &other_master_webhooks,
        &mut botcom_sended,
    )
    .await?;

    ctx.say("拡散設定リクエストを送信しました").await?;
    logged(&ctx, "拡散設定リクエストを送信しました").await?;

    Ok(())
}

/// ut_times_ubiqui_setting_recvのために必要なデータを詰める関数
async fn get_data_for_ut_times_ubiqui_setting_recv(
    _ctx: &serenity::Context,
    data: &Data,
    times_ubiqui_setting: &TimesUbiquiSettingSend,
) -> Result<(Webhook, OwnTimesData, OwnServerData)> {
    let src_member_id = times_ubiqui_setting.src_member_id;

    let db = data.connection.clone();

    // 返送先のOtherServerData
    let recv_master_webhook_url = times_ubiqui_setting.src_master_webhook_url.clone();
    let http = Http::new("");
    let recv_master_webhook = Webhook::from_url(&http, &recv_master_webhook_url).await?;

    // a_member_id と紐づいているtimeswebhookを取得
    let own_times_data = OwnTimesData::db_read(db.as_ref(), src_member_id)?
        .context("own_times_dataが登録されていません")?;

    // 自身のサーバ情報を取得
    let own_server_data =
        OwnServerData::db_read(db.as_ref())?.context("own_server_dataが登録されていません")?;

    Ok((recv_master_webhook, own_times_data, own_server_data))
}

pub async fn velify(signed_token: &str, src_guild_id: u64, data: &Data) -> Result<Claims> {
    // dbから対象サーバの情報を取得
    let db = data.connection.clone();
    let other_server_data = OtherServerData::db_read_from_guild_id(db.as_ref(), src_guild_id)?;

    // info!("public_key_pem_hashmap: {:?}", &public_key_pem_hashmap);
    // let public_key_pem = public_key_pem_hashmap
    //     .get(&src_guild_id)
    //     .context(format!("公開鍵が登録されていません. src_guild_id:{}", src_guild_id))?;

    let public_key_pem = other_server_data.public_key_pem;

    // info!("public_key_pem_hashmap: {:?}", &public_key_pem_hashmap);

    // let a = new_message.content

    // メッセージの内容を検証．検証できない場合はエラー
    let token = decode::<Claims>(
        signed_token,
        &DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).unwrap(),
        &Validation::new(Algorithm::RS256),
    )?;

    info!("token: {:?}", &token);

    Ok(token.claims)
}

/// 拡散設定リクエストを受信したときの処理
pub async fn times_ubiqui_setting_recv(
    ctx: &serenity::Context,
    data: &Data,
    signed_token: &str,
    bot_com_msg: &BotComMessage,
) -> Result<()> {
    // 検証してClaimを取り出す
    let claim = velify(signed_token, bot_com_msg.src_guild_id, data).await?;

    // 必要なデータを取得
    let (recv_master_webhook, member_times_data, own_server_data) =
        get_data_for_ut_times_ubiqui_setting_recv(ctx, data, &claim.times_ubiqui_setting_send)
            .await?;

    if own_server_data.guild_id != bot_com_msg.dst_guild_id {
        return Err(anyhow!("err guild_idが一致しません"));
    }

    let times_webhook_url = member_times_data.webhook_url;
    let times_channel_id = member_times_data.channel_id;

    // データをTimesUbiquiSettingRecvに詰める
    let times_ubiqui_setting_recv = TimesUbiquiSettingRecv {
        src_member_id: member_times_data.member_id,
        dst_guild_id: own_server_data.guild_id,
        dst_channel_id: times_channel_id,
        dst_webhook_url: times_webhook_url,
    };

    // データを詰めてシリアライズ
    // dst_guild_idにはリクエストで受け取ったsrc_guild_idを詰める
    // ここでは署名を使わないので，tokenはNone
    let bot_com_msg = BotComMessage::new(
        own_server_data.guild_id,
        bot_com_msg.src_guild_id,
        CmdKind::TimesUbiquiSettingRecv(times_ubiqui_setting_recv),
    );
    let serialized_msg = serde_json::to_string(&bot_com_msg)?;

    // データを送信
    recv_master_webhook
        .execute(ctx, false, |w| w.content(serialized_msg.to_string()))
        .await?;

    logged_serenity_ctx(
        ctx,
        &own_server_data.master_webhook_url,
        "拡散設定リクエスト 受信 返信を送信",
    )
    .await?;

    Ok(())
}

async fn get_data_for_ut_times_ubiqui_setting_set(data: &Data) -> Result<OwnServerData> {
    let connection = data.connection.clone();

    // 自身のサーバ情報を取得
    let own_server_data = OwnServerData::db_read(connection.as_ref())?
        .context("own_server_dataが登録されていません")?;

    Ok(own_server_data)
}

/// 拡散設定返信を受信したときの処理
pub async fn times_ubiqui_setting_set(
    ctx: &serenity::Context,
    data: &Data,
    times_ubiqui_setting_recv: &TimesUbiquiSettingRecv,
    _bot_com_msg: &BotComMessage,
) -> Result<()> {
    let own_server_data = get_data_for_ut_times_ubiqui_setting_set(data).await?;

    // リクエストを送信した先以外からメッセージが来ている場合はエラーとして処理する
    let botcom_sended = data.botcom_sended.write().await;
    let _sended_servers = botcom_sended.get(&times_ubiqui_setting_recv.src_member_id).ok_or(anyhow!("invalid botcom ubiquitous setting request reply 無効な拡散設定リクエスト返信です. そのユーザidからは，そのguild_idに対してリクエストを送信していません"))?;

    // sended_servers.remove(times_ubiqui_setting_send.)

    let db = data.connection.clone();

    let src_member_id = times_ubiqui_setting_recv.src_member_id;
    // other_server_dataを取得
    let dst_guild_id = times_ubiqui_setting_recv.dst_guild_id;
    let src_server_data = OtherServerData::db_read_from_guild_id(db.as_ref(), dst_guild_id)?;

    // 必要なデータをOtherTimesDataに詰める
    let _member_webhook = OtherTimesData::new(
        src_member_id,
        &src_server_data.server_name,
        times_ubiqui_setting_recv.dst_guild_id,
        times_ubiqui_setting_recv.dst_channel_id,
        &times_ubiqui_setting_recv.dst_webhook_url,
    );

    let db = data.connection.clone();

    info!("times_ubiqui_setting_set: DB処理 到達");
    src_server_data.db_upsert(db.as_ref())?;

    let msg = format!(
        "拡散設定リクエスト返信 受信\n サーバ名: {} サーバid: {} をメンバーid: {} の拡散先に設定しました",
        &src_server_data.server_name,
        &src_server_data.guild_id,
        &src_member_id
    );

    logged_serenity_ctx(ctx, &own_server_data.master_webhook_url, &msg).await?;

    Ok(())
}
