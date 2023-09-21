#![warn(clippy::str_to_string)]
use anyhow::Error;
use poise::{serenity_prelude as serenity, Event};
use sqlx::SqlitePool;
use std::{
    env::{self, var},
    sync::{Arc},
    time::Duration,
};
use tracing::info;

use commands::member_webhook::auto::{
    ut_times_set, ut_times_show, ut_times_ubiqui_setting_send, ut_times_unset,
};
use commands::member_webhook::manual::{
    ut_delete, ut_list, ut_member_webhook_reg_manual, ut_times_release,
};
use commands::{
    master_webhook::manual::{ut_serverlist, ut_set_other_masterhook, ut_set_own_master_webhook},
    member_webhook::auto,
};

use commands::member_webhook::auto::*;
use commands::types::global_data::Data;
use commands::types::botcom::CmdKind;

/// poise公式リポジトリのサンプルコードの改造
/// コメントをグーグル翻訳にかけている

// Types used by all command functions
// すべてのコマンド関数で使用される型
// type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
// pub struct Data {
//     votes: Mutex<HashMap<String, u32>>,
//     poise_mentions: AtomicU32,
//     connection: Arc<SqlitePool>,
// }

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
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
async fn event_handler(
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
            let bot_com_msg = match bot_com_msg_recv(new_message).await {
                Some(t) => t,
                None => return Ok(()),
            };

            let cmd_kind = &bot_com_msg.cmd;

            match cmd_kind {
                CmdKind::TimesUbiquiSettingSend(t) => {
                    let src_server_name = bot_com_msg.src;
                    auto::times_ubiqui_setting_recv(ctx, data, &src_server_name, t).await?;
                }
                CmdKind::TimesUbiquiSettingRecv(t) => {
                    let src_server_name = bot_com_msg.src;
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

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();
    // tracing_subscriber::fmt()
    //         .with_max_level(tracing::Level::DEBUG)
    //         .init();

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    // FrameworkOptions には、poise のすべての構成オプションが 1 つの構造体に含まれています
    // すべてのオプションを省略してデフォルト値を使用できます。by google translate
    let options = poise::FrameworkOptions {
        // ここでコマンドを登録する
        // コマンド名は1~32文字じゃないとダメみたい
        // どうやらスネークケースじゃないとだめのようだ
        commands: vec![
            help(),
            ut_set_own_master_webhook(),
            ut_set_other_masterhook(),
            ut_serverlist(),
            // ut_get_master_hook(),
            ut_member_webhook_reg_manual(),
            ut_list(),
            ut_delete(),
            ut_times_release(),
            ut_times_set(),
            ut_times_unset(),
            ut_times_show(),
            ut_times_ubiqui_setting_send(),
        ],

        // ここでprefixを設定する
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot"),
                poise::Prefix::Literal("hey bot,"),
            ],
            ..Default::default()
        },

        /// The global error handler for all error cases that may occur
        /// 発生する可能性のあるすべてのエラーケースに対応するグローバルエラーハンドラー
        on_error: |error| Box::pin(on_error(error)),

        /// This code is run before every command
        /// このコードはすべてのコマンドの前に実行されます
        /// serenityでフレームワークに.bafore()を登録するみたいな感じと推測
        pre_command: |ctx| {
            Box::pin(async move {
                info!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        /// This code is run after a command if it was successful (returned Ok)
        /// このコードは、コマンドが成功した場合 (Ok が返された場合)、コマンドの後に実行されます。
        /// serenityでフレームワークに.after()を登録するみたいな感じと推測
        post_command: |ctx| {
            Box::pin(async move {
                info!("Executed command {}!", ctx.command().qualified_name);
            })
        },

        /// Every command invocation must pass this check to continue execution
        /// 実行を続行するには、すべてのコマンド呼び出しがこのチェックに合格する必要があります
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),

        /// Enforce command checks even for owners (enforced by default)
        /// Set to true to bypass checks, which is useful for testing
        /// 所有者に対してもコマンド チェックを強制します (デフォルトで強制)
        /// チェックをバイパスするには true に設定します。これはテストに役立ちます
        skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                info!("Got an event in event handler: {:?}", event.name());
                event_handler(_ctx, event, _framework, _data).await
            })
        },
        ..Default::default()
    };

    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    poise::Framework::builder()
        .token(
            var("DISCORD_TOKEN")
                .expect("Missing `DISCORD_TOKEN` env var, see README for more information."),
        )
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    connection: Arc::new(pool),
                })
            })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged()
                | serenity::GatewayIntents::MESSAGE_CONTENT
                | serenity::GatewayIntents::GUILD_WEBHOOKS,
        )
        .run()
        .await
        .unwrap();
}
