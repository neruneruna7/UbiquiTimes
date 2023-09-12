use tracing::{error, info};

use crate::*;

#[allow(non_snake_case)]
#[poise::command(prefix_command, track_edits, slash_command)]
async fn UTregserver(
    ctx: Context<'_>, 
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "拡散先サーバのギルド（サーバー）ID"] guild_id: Option<i64>,
) -> Result<()> {
    // msg.contentを分割して、server_nameとwebhook_urlを取得する
    // let mut iter = msg.content.split_whitespace();
    // let _ = iter.next().unwrap();
    // let server_name = iter.next().unwrap();
    // let guild_id = iter.next().unwrap().parse::<i64>().unwrap();
    // let webhook_url = iter.next().unwrap();

    // log
    info!("server_name: {}, webhook_url: {}, guild_id: {:?}", server_name, master_webhook_url, guild_id);

    // DBに登録する
    let connection = ctx.data().connection.clone();


    master_webhook_insert(
        connection.as_ref(), 
        MasterWebhook::from(None, &server_name, guild_id, &master_webhook_url)
    ).await?;

    Ok(())
}

#[allow(non_snake_case)]
#[poise::command(prefix_command, track_edits, slash_command)]
async fn getMasterHook(
    ctx: Context<'_>,
    #[description = "webhookを確認するサーバ名"] server_name: String,
) -> Result<()> {
    // log
    info!("server_name: {}", server_name);

    // DBから取得する
    let connection = ctx.data().connection.clone();

    let master_webhook = master_webhook_select(
        connection.as_ref(), &server_name).await?;

    ctx.say(format!("master_webhook: {:?}", master_webhook))
        .await?;

    Ok(())
}


// 自動でメンバーwebhookを登録できるようにしたい
// // メンバーwebhookを登録する
// #[allow(non_snake_case)]
// #[poise::command(prefix_command, track_edits, slash_command)]
// async fn UTregister(
//     ctx: Context<'_>, 
//     #[description = "拡散先のサーバ名"] server_name: String,
//     #[description = "拡散先のチャンネルID"] channel_id: i64,
// ) -> Result<()> {
//     // もしチャンネルにwebhookが存在していたら、それを使う
//     // なければ、新規に作成する
//     // チャンネルidから，存在しているwebhookを取得する
//     let webhooks = msg.channel_id.webhooks(&ctx).await?;


    
    
//     // UT- username という名前のwebhookがあるかどうか
//     let webhook = if let Some(webhook) = webhooks.iter().find(|w| w.name == Some(format!("UT-{}", &msg.author.name))) {
//         webhook.to_owned()
//     } else {
//         msg.channel_id.create_webhook(&ctx, format!("UT-{}", &msg.author.name)).await?
//     };

//     let my_webhook_url = webhook.url()?;

//     // さらなる記述が必要

//     Ok(())
// }


// 手動でメンバーwebhookを登録する
// (prefix)UTregisterM server_name webhook_url 
#[allow(non_snake_case)]
#[poise::command(prefix_command, track_edits,aliases("UTregisterM"), slash_command)]
async fn UT_member_webhook_register_manual(
    ctx: Context<'_>, 
    #[description = "拡散先のサーバ名"] server_name: String,
    #[description = "拡散先のチャンネルID"] channel_id: i64,
    #[description = "拡散先チャンネルのwebhook URL"] webhook_url: String,
) -> Result<()> {
    let connection = ctx.data().connection.clone();


    let menber_webhook = MemberWebhook {
        id: None,
        server_name,
        member_id: 1234,
        channel_id,
        webhook_url,
    };

    member_webhook_insert(connection.as_ref(), menber_webhook).await?;

    info!("member webhook inserted");

    Ok(())
}