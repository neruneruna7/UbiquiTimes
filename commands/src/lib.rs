
pub mod error;
pub mod global_data;
pub mod help_command;
pub mod poise_commands;





// /// 現在エラー発生中 master_webhook_urlがdataに無いと予測
// // 一旦コメントアウト
// // discordにもログを流したい
// async fn logged(_ctx: &Context<'_>, _msg: &str) -> Result<()> {
//     // let master_webhook_url = ctx.data().master_webhook_url.read().await;

//     // let webhook = Webhook::from_url(ctx, &master_webhook_url)
//     //     .await
//     //     .context(format!(
//     //         "globaldataのmaster_webhook_urlに異常があるか，登録されていません． url: {}",
//     //         &master_webhook_url
//     //     ))?;

//     // info!(msg);
//     // webhook.execute(&ctx, false, |w| w.content(msg)).await?;

//     Ok(())
// }

// /// serenityのctxだとctx.sayが使えないので
// async fn logged_serenity_ctx(
//     ctx: &serenity::Context,
//     master_webhook_url: &str,
//     msg: &str,
// ) -> Result<()> {
//     let my_webhook = Webhook::from_url(&ctx, master_webhook_url).await?;

//     info!(msg);
//     let builder = ExecuteWebhook::new().content(msg);
//     my_webhook.execute(ctx, false, builder).await?;
//     Ok(())
// }
