use anyhow::Error;
use poise::{serenity_prelude as serenity, Event};

use commands::global_data::Data;
use commands::member_webhook::auto;
use commands::member_webhook::auto::*;
use commands::types::botcom::CmdKind;
use tracing::info;

/// poise公式リポジトリのサンプルコードの改造
/// コメントをグーグル翻訳にかけている

// Types used by all command functions
// すべてのコマンド関数で使用される型
// type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
// pub struct Data {
//     votes: Mutex<HashMap<String, u32>>,
//     poise_mentions: AtomicU32,
//     connection: Arc<SqlitePool>,
// }

pub async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    // これはカスタム エラー ハンドラーです
    // 多くのエラーが発生する可能性があるため、カスタマイズしたいエラーのみを処理します
    // そして残りをデフォルトのハンドラーに転送します
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            info!("Error in command `{}`: {:?}", ctx.command().name, error,);
            ctx.say(error.to_string()).await.ok();
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                info!("Error while handling error: {}", e)
            }
        }
    }
}

// イベントハンドラ
// serenityの，EventHadlerトレイトを実装して実現していたものと同等と推測
pub async fn event_handler(
    ctx: &serenity::Context,
    event: &Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        Event::Ready { data_about_bot } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        Event::Message { new_message } => {
            println!("msg recvd");

            // info!("Got a message from a bot: {:?}", new_message);
            let token = match bot_com_msg_recv(new_message, data).await? {
                Some(t) => t,
                None => return Ok(()),
            };

            let claims = token.claims;

            let cmd_kind = &claims.cmdkind;

            match cmd_kind {
                CmdKind::TimesUbiquiSettingSend(t) => {
                    let src_guild_id = claims.sub;
                    let src_server_name = claims.iss;
                    auto::times_ubiqui_setting_recv(ctx, data, src_guild_id, &src_server_name, t)
                        .await?;
                }
                CmdKind::TimesUbiquiSettingRecv(t) => {
                    let src_server_name = claims.iss;
                    auto::times_ubiqui_setting_set(ctx, data, &src_server_name, t).await?;
                }
                CmdKind::None => {}
            }

            // if new_message.content.to_lowercase().contains("poise") {
            //     let mentions = data.poise_mentions.load(Ordering::SeqCst) + 1;
            //     data.poise_mentions.store(mentions, Ordering::SeqCst);
            //     new_message
            //         .reply(ctx, format!("Poise has been mentioned {} times", mentions))
            //         .await?;
            // }
        }
        _ => {}
    }
    Ok(())
}
