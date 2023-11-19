use anyhow::Error;
use commands::bot_communicate::send::{
    bot_com_msg_recv, times_ubiqui_setting_recv, times_ubiqui_setting_set,
};
use poise::{serenity_prelude as serenity, Event};

use commands::bot_communicate::CmdKind;
use commands::global_data::Data;
use commands::register_masterhook_ctx_data;

use tracing::info;

// use commands::member_webhook::auto::{
//     ut_times_set, ut_times_show, ut_times_ubiqui_setting_send, ut_times_unset,
// };
// use commands::member_webhook::manual::{
//     ut_delete, ut_list, ut_member_webhook_reg_manual, ut_times_release,
// };
use commands::own_server::times::{ut_times_set, ut_times_show, ut_times_unset};
// use commands::member_webhook::manual::{
//     ut_delete, ut_list, ut_member_webhook_reg_manual, ut_times_release,
// };

use commands::other_server::times::{
    ut_delete, ut_list, ut_member_webhook_reg_manual, ut_times_release,
};

use commands::{
    bot_communicate::send::ut_times_ubiqui_setting_send,
    other_server::server::{ut_delete_other_masterhook, ut_serverlist, ut_set_other_server_data},
    own_server::server::{ut_get_own_server_data, ut_set_own_server_data},
};

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

// ヘルプコマンドだけメインに記述してしまうことにした
/// ヘルプを表示します
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> anyhow::Result<()> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

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
            register_masterhook_ctx_data(&data.connection, data).await?;
        }
        Event::Message { new_message } => {
            println!("msg recvd");

            // info!("Got a message from a bot: {:?}", new_message);
            let bot_com_msg = match bot_com_msg_recv(new_message, data).await {
                Some(t) => t,
                None => return Ok(()),
            };

            match &bot_com_msg.cmd_kind {
                CmdKind::TimesUbiquiSettingSendToken(t) => {
                    times_ubiqui_setting_recv(ctx, data, t, &bot_com_msg).await?;
                }
                CmdKind::TimesUbiquiSettingRecv(t) => {
                    times_ubiqui_setting_set(ctx, data, t, &bot_com_msg).await?;
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

// Shuttleとセルフホストの両方で使えるようにするため，切り出している
pub fn commands_vec() -> Vec<poise::Command<Data, Error>> {
    vec![
        help(),
        ut_set_own_server_data(),
        ut_get_own_server_data(),
        ut_set_other_server_data(),
        ut_serverlist(),
        ut_delete_other_masterhook(),
        // ut_get_master_hook(),
        ut_member_webhook_reg_manual(),
        ut_list(),
        ut_delete(),
        ut_times_release(),
        ut_times_set(),
        ut_times_unset(),
        ut_times_show(),
        ut_times_ubiqui_setting_send(),
    ]
}
