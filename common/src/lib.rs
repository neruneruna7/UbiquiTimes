use anyhow::Error;
use commands::bot_message_communicator::req_receiver::WebhookReqReceiver;
use commands::bot_message_communicator::res_receiver::WebhookResReceiver;
use commands::poise_commands::setting_commands::{
    member_setting_commands, server_setting_commands,
};

use commands::bot_message_communicator::MultiReceiver;
use commands::poise_commands::spreading_commands;
use poise::serenity_prelude::{self as serenity, FullEvent};

use commands::global_data::Data;

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
        poise::FrameworkError::Command { error, ctx, .. } => {
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
    event: &FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        FullEvent::Message { new_message } => {
            info!("new message: {:?}", new_message);

            // info!("Got a message from a bot: {:?}", new_message);
            // この辺ややこしいことになってるので要改善
            let is_bot = WebhookReqReceiver::check(new_message);
            if !is_bot {
                info!("Not a bot message");
                return Ok(());
            }
            info!("Bot message");

            let webhook_receiver = MultiReceiver::new(WebhookReqReceiver, WebhookResReceiver);

            info!("receiver start");
            webhook_receiver.receiv(new_message, ctx, framework).await?;
            info!("receiver done");

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
        server_setting_commands::ut_initialize(),
        server_setting_commands::ut_get_own_server_data(),
        member_setting_commands::ut_times_set(),
        member_setting_commands::ut_times_show(),
        member_setting_commands::ut_times_unset(),
        member_setting_commands::ut_times_spread_setting(),
        member_setting_commands::ut_list(),
        member_setting_commands::ut_times_spread_unset(),
        spreading_commands::ut_times_release(),
        spreading_commands::hello(),
    ]
}
