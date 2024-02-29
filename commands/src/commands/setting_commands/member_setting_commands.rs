// - メンバーのTimesがどのチャンネルなのかを設定するコマンド
// - メンバーのTimesがどのチャンネルなのかを削除するコマンド
// - メンバーのTimes拡散設定コマンド
// - メンバーのTimes拡散設定情報を取得する
// - メンバーのTimes拡散設定情報を削除する
// - メンバーのTimes拡散設定情報を手動で作成する
//

use super::super::command_check;
use crate::global_data::Context;
use crate::other_server_repository::OtherTimesRepository;
use crate::own_server::OwnTimes;
use crate::own_server_repository::OwnTimesRepository;

use anyhow::Context as anyhowContext;
use anyhow::Result;

use tracing::info;

fn create_member_webhook_name(member_id: u64) -> String {
    format!("UT-{}", member_id)
}

/// そのサーバーでの自分のtimesであることをセットする
///
/// 本サーバにおいて，このコマンドを実行したチャンネルがあなたのTimesであるとbotに登録します．
/// 結果は実行するチャンネルに依存します．
#[poise::command(prefix_command, track_edits, aliases("UtTimesSet"), slash_command)]
pub async fn ut_times_set(
    ctx: Context<'_>,
    #[description = "拡散時に使う名前を入力してください"] name: String,
) -> Result<()> {
    let member_id = ctx.author().id.0;
    let member_name = name;
    let channel_id = ctx.channel_id().0;

    let webhook_name = Some(create_member_webhook_name(member_id));

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

    // 自身のTimes情報を保存
    let own_times = OwnTimes::new(member_id, &member_name, channel_id, &webhook_url);
    let own_times_repository = ctx.data().own_times_repository.clone();
    own_times_repository.upsert(own_times).await?;

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
    // untimesと入力されてないときは弾く
    // 誤操作の防止
    command_check(&ctx, &untimes, "untimes").await?;

    let member_id = ctx.author().id.0;

    // webhookを削除
    // 想定通りに動くのか未検証
    let webhooks = ctx.channel_id().webhooks(&ctx).await?;
    let webhook_name = Some(create_member_webhook_name(member_id));
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



/// あなたのメンバー拡散先リストを表示します
///
/// あなたのメンバーウェブフックを登録しているサーバー名を，一覧表示します
#[poise::command(prefix_command, track_edits, aliases("UTlist"), slash_command)]
pub async fn ut_list(ctx: Context<'_>) -> Result<()> {
    let member_id = ctx.author().id.0;

    // let db = ctx.data().connection.clone();
    // let other_times_data_vec = OtherTimesData::db_read_from_member_id(db.as_ref(), member_id)?;
    let other_times_repository = ctx.data().other_times_repository.clone();
    let other_times_data_vec = other_times_repository.get_from_member_id(member_id).await?;

    let mut response = String::new();
    response.push_str("拡散先リスト\n --------- \n```");

    for other_times_data in other_times_data_vec {
        response.push_str(&format!("{}\n", other_times_data.dst_server_name));
    }
    response.push_str("```");

    ctx.say(response).await?;

    Ok(())
}
