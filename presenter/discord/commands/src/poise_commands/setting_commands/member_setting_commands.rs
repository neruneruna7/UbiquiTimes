// - メンバーのTimesがどのチャンネルなのかを設定するコマンド
// - メンバーのTimesがどのチャンネルなのかを削除するコマンド
// - メンバーのTimesを指定したサーバに拡散設定するコマンド
// - メンバーのTimes拡散設定情報を取得する
// - メンバーのTimes拡散設定情報を削除する
// - メンバーのTimes拡散設定情報を手動で作成する
//

use super::super::command_check;
use crate::global_data::Context;

use anyhow::Context as anyhowContext;
use anyhow::Result;

use domain::functions::add_prefix_memberid;
use domain::models::communication::TimesSettingRequest;

use domain::models::guild_data::OwnTimes;
use domain::tracing::info;
use domain::traits::{communicators::*, repositorys::*};
use message_communicator::request_sender::PoiseWebhookReqSender;
use poise::serenity_prelude::CreateWebhook;

/// そのサーバーでの自分のtimesであることをセットする
///
/// 本サーバにおいて，このコマンドを実行したチャンネルがあなたのTimesであるとbotに登録します．
/// 結果は実行するチャンネルに依存します．
#[poise::command(prefix_command, track_edits, aliases("UtTimesSet"), slash_command)]
pub async fn ut_times_set(
    ctx: Context<'_>,
    #[description = "拡散時に使う名前を入力してください"] name: String,
) -> Result<()> {
    let member_id = ctx.author().id.get();
    let member_name = name;
    let channel_id = ctx.channel_id().get();

    let webhook_name = add_prefix_memberid(member_id);

    // チャンネルに"UT-{メンバーid}"のwebhookがあるか確認
    let webhooks = ctx.channel_id().webhooks(&ctx).await?;

    let webhook_exists = webhooks.iter().any(|webhook| {
        // webhook.name == webhook_name_option
        webhook.name == Some(webhook_name.clone())
    });

    // 存在するならそれを返す，無ければ作る
    let webhook = if webhook_exists {
        info!("member webhook exists");
        webhooks
            .iter()
            .find(|webhook| webhook.name == Some(webhook_name.clone()))
            .unwrap()
            .clone()
    } else {
        info!("member webhook not exists. create new webhook");
        let builder = CreateWebhook::new(webhook_name);
        ctx.channel_id()
            .create_webhook(&ctx, builder)
            .await
            .context("webhookの作成に失敗しました")?
    };

    info!("create now member_webhook_url{:?}", webhook.url()?);

    let webhook_url = webhook.url()?;

    // 自身のTimes情報を保存
    let own_times = OwnTimes::new(member_id, &member_name, channel_id, &webhook_url);
    let own_times_repository = ctx.data().own_times_repository.clone();
    own_times_repository.upsert(own_times).await?;

    ctx.say("このチャンネルを，本サーバでのあなたのTimesとして登録しました")
        .await?;

    Ok(())
}

/// 自身のtimesを表示する
///
/// 本サーバにおいて，あなたの登録されているTimesの情報を表示します．
/// 結果は実行するチャンネルに依存しません．
#[poise::command(prefix_command, track_edits, aliases("UtTimesShow"), slash_command)]
pub async fn ut_times_show(ctx: Context<'_>) -> Result<()> {
    let member_id = ctx.author().id.get();

    let own_times_repository = ctx.data().own_times_repository.clone();
    let own_times = own_times_repository.get(member_id).await?;

    let own_times = match own_times {
        Some(own_times) => own_times,
        None => {
            ctx.say("あなたのTimesが登録されていません").await?;
            return Ok(());
        }
    };

    let response = format!(
        "Times情報\n --------- \n```name: {}\nchannel_id: {}\nwebhook_url: {}```",
        own_times.member_name, own_times.channel_id, own_times.times_webhook_url
    );

    ctx.say(response).await?;

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
    // untimesと入力されてないときは弾く
    // 誤操作の防止
    command_check(&ctx, &untimes, "untimes").await?;

    let member_id = ctx.author().id.get();

    // webhookを削除
    // 想定通りに動くのか未検証
    let webhooks = ctx.channel_id().webhooks(&ctx).await?;
    let webhook_name = Some(add_prefix_memberid(member_id));
    let webhook = webhooks
        .iter()
        .find(|webhook| webhook.name == webhook_name)
        .context("webhookが見つかりませんでした")?;
    webhook.delete(&ctx).await?;

    // 自身のTimes情報を削除
    let own_times_repository = ctx.data().own_times_repository.clone();
    own_times_repository.delete(member_id).await?;

    ctx.say("本サーバでのあなたのTimes登録を削除しました")
        .await?;

    Ok(())
}

/// 指定したサーバに大して，あなたのTimesを拡散できるように設定します
///
/// ここで入力するサーバ名は，必ずしも一意である必要はありません
/// 人間がその名前をみて，どのサーバかを判断できるのであれば問題ありません
#[poise::command(
    prefix_command,
    track_edits,
    aliases("UtTimesSpreadSet"),
    slash_command
)]
pub async fn ut_times_spread_setting(
    ctx: Context<'_>,
    #[description = "拡散先サーバのギルド（サーバー）ID"] dst_guild_id: String,
    #[description = "拡散先の識別用サーバ名"] dst_server_name: String,
) -> Result<()> {
    // slash commandではu64型をうまく受け取れないので，Stringで受け取ってから変換する
    let dst_guild_id = dst_guild_id.parse::<u64>()?;

    // リクエストメッセージを組み立てる
    // 自身のサーバ情報が必要なので，それを取得する
    let own_guild_repository = ctx.data().own_server_repository.clone();
    let own_guild = own_guild_repository.get().await?;

    let own_times_repository = ctx.data().own_times_repository.clone();
    let own_times = own_times_repository.get(ctx.author().id.get()).await?;

    let own_times = match own_times {
        Some(own_times) => own_times,
        None => {
            ctx.say("あなたのTimesが登録されていません").await?;
            return Ok(());
        }
    };

    // let dst_guild = OtherGuild::new(dst_guild_id, &dst_server_name, webhook_url, public_key_pem);

    // リクエストメッセージのもととなるデータを作成
    let times_setting_req = TimesSettingRequest::new(
        ctx.author().id.get(),
        own_guild.manage_webhook_url.clone(),
        ctx.channel_id().get(),
        own_times.times_webhook_url.clone(),
    );

    let member_id = ctx.author().id.get();
    let sent_member_and_guild_ids = ctx.data().sent_member_and_guild_ids.clone();

    let ca_driver = ctx.data().ca_driver.clone();
    // 設定リクエストを送信する
    let req_sender = PoiseWebhookReqSender::new(ca_driver);

    req_sender
        .times_setting_request_send(
            &own_guild,
            dst_guild_id,
            &dst_server_name,
            member_id,
            times_setting_req,
            sent_member_and_guild_ids,
        )
        .await?;

    ctx.say("設定リクエストを送信しました").await?;

    Ok(())
}

/// 拡散先サーバのリストを表示します
///
/// あなたのTimesを拡散しているサーバのリストを表示します
#[poise::command(prefix_command, track_edits, aliases("UTlist"), slash_command)]
pub async fn ut_list(ctx: Context<'_>) -> Result<()> {
    let member_id = ctx.author().id.get();

    // let db = ctx.data().connection.clone();
    // let other_times_data_vec = OtherTimesData::db_read_from_member_id(db.as_ref(), member_id)?;
    let other_times_repository = ctx.data().other_times_repository.clone();
    let other_times_data_vec = other_times_repository.get_from_member_id(member_id).await?;

    let mut response = String::new();
    response.push_str("拡散先リスト\n --------- \n```");

    for other_times_data in other_times_data_vec {
        response.push_str(&format!("{}\n", other_times_data.dst_guild_name));
    }
    response.push_str("```");

    ctx.say(response).await?;

    Ok(())
}

// Times拡散設定情報を削除する
#[poise::command(
    prefix_command,
    track_edits,
    aliases("UtTimesSpreadUnset"),
    slash_command
)]
pub async fn ut_times_spread_unset(
    ctx: Context<'_>,
    #[description = "削除するサーバ名"] server_name: String,
) -> Result<()> {
    let member_id = ctx.author().id.get();

    let other_times_repository = ctx.data().other_times_repository.clone();
    other_times_repository
        .delete(&server_name, member_id)
        .await?;

    ctx.say("拡散先サーバを削除しました").await?;

    Ok(())
}
