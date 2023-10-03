use super::*;

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

/// そのサーバーでの自分のtimesであることをセットする
///
/// 本サーバにおいて，このコマンドを実行したチャンネルがあなたのTimesであるとbotに登録します．
/// 結果は実行するチャンネルに依存します．
#[poise::command(prefix_command, track_edits, aliases("UtTimesSet"), slash_command)]
pub async fn ut_times_set(
    ctx: Context<'_>,
    #[description = "拡散時に使う名前を入力してください"] name: String,
) -> Result<()> {
    // sign_str_command(&ctx, &times, "times").await?;

    let member_id = ctx.author().id.0;
    let member_name = name;
    let channel_id = ctx.channel_id().0;

    let webhook_name = Some(format!("UT-{}", member_id));

    // チャンネルに"UT-{メンバーid}"のwebhookがあるか確認
    let webhooks = ctx.channel_id().webhooks(&ctx).await?;

    let webhook_exists = webhooks.iter().any(|webhook| {
        // webhook.name == webhook_name_option
        webhook.name == webhook_name
    });

    // 存在するならそれを返す，無ければ作る
    let webhook = if webhook_exists {
        info!("member webhook exists");
        webhooks
            .iter()
            .find(|webhook| webhook.name == webhook_name)
            .unwrap()
            .clone()
    } else {
        info!("member webhook not exists. create new webhook");
        ctx.channel_id()
            .create_webhook(&ctx, webhook_name.unwrap())
            .await
            .context("webhookの作成に失敗しました")?
    };

    info!("{:?}", webhook);

    let webhook_url = webhook.url()?;
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
#[poise::command(prefix_command, track_edits, aliases("UtTimesUnset"), slash_command)]
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
#[poise::command(prefix_command, track_edits, aliases("UtTimesShow"), slash_command)]
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
