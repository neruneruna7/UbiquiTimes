#![warn(clippy::str_to_string)]

mod commands;
mod master_webhook;

use anyhow::{Error, Result};
use poise::{serenity_prelude as serenity, Event};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    env::{self, var},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tracing::info;

/// poise公式リポジトリのサンプルコードの改造
/// コメントをグーグル翻訳にかけている

// Types used by all command functions
// すべてのコマンド関数で使用される型
// type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
pub struct Data {
    votes: Mutex<HashMap<String, u32>>,
    poise_mentions: AtomicU32,
    connection: Arc<SqlitePool>,
}

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
            if new_message.content.to_lowercase().contains("poise") {
                let mentions = data.poise_mentions.load(Ordering::SeqCst) + 1;
                data.poise_mentions.store(mentions, Ordering::SeqCst);
                new_message
                    .reply(ctx, format!("Poise has been mentioned {} times", mentions))
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<()> {
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
            commands::help(),
            // commands::vote(),
            // commands::getvotes(),
            // commands::member_webhook_register_manual(),
            commands::ut_serverlist(),
            commands::ut_member_webhook_reg_manual(),
            commands::ut_list(),
            commands::ut_delete(),
            commands::ut_execute(),
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
                Ok(())
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
                    votes: Mutex::new(HashMap::new()),
                    poise_mentions: AtomicU32::new(0),
                    connection: Arc::new(pool),
                })
            })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .run()
        .await
        .unwrap();
}
