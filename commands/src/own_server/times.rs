use crate::own_server::{OwnTimesData, OwnTimesDataTable};
use crate::sign_str_command;

// use crate::{Context, Result};

use anyhow::Context as _;

use crate::db_query::SledTable;

use crate::*;

use anyhow::Context as anyhowContext;
use anyhow::Result;

use tracing::info;

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

    let own_timed_data = OwnTimesData::new(member_id, &member_name, channel_id, &webhook_url);

    let db = ctx.data().connection.clone();
    let own_times_table = OwnTimesDataTable::new(&db);
    own_times_table.upsert(&own_timed_data.member_id.to_string(), &own_timed_data);

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

    let member_id = ctx.author().id.0;

    let db = ctx.data().connection.clone();
    let own_times_table = OwnTimesDataTable::new(&db);
    own_times_table.delete(&member_id.to_string())?;

    ctx.say("本サーバでのあなたのTimes登録を削除しました")
        .await?;

    Ok(())
}

/// デバッグ用に member_times_data を全て表示する
#[poise::command(prefix_command, track_edits, aliases("UtTimesShow"), slash_command)]
pub async fn ut_times_show(ctx: Context<'_>) -> Result<()> {
    let db = ctx.data().connection.clone();
    let own_times_table = OwnTimesDataTable::new(&db);
    let own_times_data = own_times_table.read_all()?;

    let mut response = String::new();

    // スコープが小さいため，ループ変数名は名前に意味を持たせるよりも，短く見やすいものを優先した
    for t in own_times_data {
        response.push_str(&format!(
            "{}: times_channel_id: {}\n",
            t.member_name, t.member_id
        ));
    }

    ctx.say(response).await?;
    Ok(())
}
