#![warn(clippy::str_to_string)]

use common::commands_vec;
use poise::serenity_prelude as serenity;
use std::{
    collections::HashMap,
    env::{var},
    sync::Arc,
    time::Duration,
};

use poise::serenity_prelude::RwLock;


use tracing::info;

// use commands::member_webhook::auto::{
//     ut_times_set, ut_times_show, ut_times_ubiqui_setting_send, ut_times_unset,
// };
// use commands::member_webhook::manual::{
//     ut_delete, ut_list, ut_member_webhook_reg_manual, ut_times_release,
// };

// use commands::member_webhook::manual::{
//     ut_delete, ut_list, ut_member_webhook_reg_manual, ut_times_release,
// };

use commands::global_data::Data;

// poise公式リポジトリのサンプルコードの改造
// コメントをグーグル翻訳にかけている

// Types used by all command functions
// すべてのコマンド関数で使用される型
// type Error = Box<dyn std::error::Error + Send + Sync>;
// type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
// pub struct Data {
//     votes: Mutex<HashMap<String, u32>>,
//     poise_mentions: AtomicU32,
//     connection: Arc<SqlitePool>,
// }

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
        commands: commands_vec(),

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
        on_error: |error| Box::pin(common::on_error(error)),

        /// This code is run before every command
        /// このコードはすべてのコマンドの前に実行されます
        /// serenityでフレームワークに.before()を登録するみたいな感じと推測
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
                common::event_handler(_ctx, event, _framework, _data).await
                // event_handler(_ctx, event, _framework, _data).await
            })
        },
        ..Default::default()
    };

    let db = sled::open("my_database").unwrap();
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
                    connection: Arc::new(db),
                    master_webhook_url: RwLock::new(String::new()),
                    public_key_pem_hashmap: RwLock::new(HashMap::new()),
                    botcom_sended: RwLock::new(HashMap::new()),
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
