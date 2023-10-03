use super::*;
use crate::member_webhook::MemberWebhook;
use crate::sign_str_command;

use crate::global_data::Data;
use crate::loged;
use crate::{db_query::master_webhooks::master_webhook_select_all, Context, Result};

use anyhow::{anyhow, Context as _};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Http;

use poise::serenity_prelude::Webhook;

use tracing::debug;
use tracing::info;

use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

use crate::db_query::{member_webhooks, own_server_times_data::*};
use crate::db_query::{
    own_server_data::{self, *},
    own_server_times_data,
};

use crate::bot_communicate::*;
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
    let bot_com_msg: BotComMessage = match serde_json::from_str(&new_message.content) {
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

    let private_key_pem = server_data.private_key_pem;

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

    let mut claims = Claims::new(
        &server_data.server_name,
        server_data.guild_id,
        "ループ内で変更してください",
        CmdKind::TimesUbiquiSettingSend(times_ubiqui_setting_send),
    );

    // 送信するメッセージの内容を詰める
    // ここでは,cmdkindはNoneのまま
    // tokenはループ内で詰める
    let mut bot_com_msg = BotComMessage::new(server_data.guild_id, 0, None, None);

    info!("craims: {:?}", &claims);

    // claims作成
    // botcommsg作成
    // claims署名 tokenに
    // botcommsgのdst_guild__idとtokenをループ内で書き換え
    // botcommsgをシリアライズ
    // 送信

    // ここのhttpはどうするか，空白トークンのHttpをnewするか，ctxを使うか
    for other_master_webhook in other_master_webhooks.iter() {
        let webhook = Webhook::from_url(&ctx, &other_master_webhook.webhook_url).await?;
        // bot_com_msg.dst = other_master_webhook.server_name.clone();
        claims.aud = other_master_webhook.server_name.clone();

        // ここでサインする
        let token = encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?,
        )
        .context("署名に失敗")?;

        // 書き換えるべき値をここで書き換え
        bot_com_msg.dst_guild_id = other_master_webhook.guild_id;
        bot_com_msg.token = Some(token);

        let serialized_msg = serde_json::to_string(&bot_com_msg)?;

        webhook
            .execute(&ctx, false, |w| w.content(serialized_msg))
            .await?;
    }

    ctx.say("拡散設定リクエストを送信しました").await?;

    loged(&ctx, "拡散設定リクエストが送信されました").await?;

    Ok(())
}

/// 拡散設定リクエストを受信したときの処理
pub async fn times_ubiqui_setting_recv(
    ctx: &serenity::Context,
    data: &Data,
    src_guild_id: u64,
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
    let own_server_data =
        own_server_data::select_own_server_data_without_guild_id(connection.as_ref()).await?;

    // データをTimesUbiquiSettingRecvに詰める
    let times_ubiqui_setting_recv = TimesUbiquiSettingRecv {
        src_member_id,
        dst_guild_id: own_server_data.guild_id,
        dst_channel_id: times_channel_id,
        dst_webhook_url: times_webhook_url,
    };

    // let claims = Claims::new(
    //     src_server_name,
    //     src_guild_id,
    //     &own_server_data.server_name,
    //     CmdKind::TimesUbiquiSettingRecv(times_ubiqui_setting_recv),
    // );

    // // 署名
    // let token = encode(
    //     &Header::new(Algorithm::RS256),
    //     &claims,
    //     &EncodingKey::from_rsa_pem(own_server_data.private_key_pem.as_bytes())?,
    // )
    // .context("署名に失敗")?;

    // データをシリアライズ
    // ここにおいては，srcとdstがそのほかの構造体と逆になる
    // つまり，自身のサーバがsrcである
    // ここでは署名を使わないので，tokenはNone
    let bot_com_msg = BotComMessage::new(
        own_server_data.guild_id,
        src_guild_id,
        None,
        Some(CmdKind::TimesUbiquiSettingRecv(times_ubiqui_setting_recv)),
    );

    let serialized_msg = serde_json::to_string(&bot_com_msg)?;

    // データを送信
    recv_master_webhook
        .execute(ctx, false, |w| w.content(serialized_msg.to_string()))
        .await?;

    let my_webhook = Webhook::from_url(&http, &own_server_data.master_webhook_url).await?;
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
