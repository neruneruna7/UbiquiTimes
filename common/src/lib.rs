use anyhow::Error;
use commands::help_command::help;
use commands::poise_commands::setting_commands::{
    member_setting_commands, server_setting_commands,
};

use commands::poise_commands::spreading_commands;
use domain::traits::communicators::*;
use message_communicator::request_receiver::PoiseWebhookReqReceiver;
use message_communicator::response_receiver::PoiseWebhookResReceiver;
use poise::serenity_prelude::{self as serenity, FullEvent};

use commands::global_data::Data;

use domain::tracing::info;

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
    _ctx: &serenity::Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        FullEvent::Message { new_message } => {
            info!("new message: {:?}", new_message);

            // // info!("Got a message from a bot: {:?}", new_message);
            // // この辺ややこしいことになってるので要改善
            // let is_bot = WebhookReqReceiver::check(new_message);
            // if !is_bot {
            //     info!("Not a bot message");
            //     return Ok(());
            // }
            // info!("Bot message");

            // let webhook_receiver = MultiReceiver::new(WebhookReqReceiver, WebhookResReceiver);
            let ca_driver = _data.ca_driver.clone();
            let own_times_repository = _data.own_times_repository.clone();
            let own_guild_id = new_message.guild_id.unwrap().get();

            let req_receiver = PoiseWebhookReqReceiver::new(ca_driver, own_times_repository);
            req_receiver
                .times_setting_receive_and_response(new_message, own_guild_id)
                .await?;

            let other_times_repository = _data.other_times_repository.clone();
            let _res_receiver = PoiseWebhookResReceiver::new(other_times_repository);

            // info!("receiver start");
            // webhook_receiver.receiv(new_message, ctx, framework).await?;
            // info!("receiver done");

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
