use std::collections::HashMap;

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Error};
use commands::other_server_repository::{SledOtherServerRepository, SledOtherTimesRepository};
use commands::own_server_repository::{SledOwnServerRepository, SledOwnTimesRepository};
use commands::sign::keys_gen::RsaKeyGenerator;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_secrets::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use tokio::sync::RwLock;

use commands::global_data::{Context, Data};

use tracing::info;

// struct Data {} // User data, which is stored and accessible in all command invocations
// type Error = Box<dyn std::error::Error + Send + Sync>;
// type Context<'a> = poise::Context<'a, Data, Error>;

const MODE: &str = "debug1";

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    // #[shuttle_static_folder::StaticFolder(folder = "db")] static_folder: PathBuf,
) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    // tracing_subscriber::fmt::init();

    let (discord_token, db) = if MODE == "debug1" {
        let discord_token = secret_store
            .get("DISCORD_TOKEN")
            .context("'DISCORD_TOKEN' was not found")?;
        let db = sled::open("my_database").unwrap();
        (discord_token, db)
    } else if MODE == "debug2" {
        let discord_token = secret_store
            .get("DISCORD_TOKEN2")
            .context("'DISCORD_TOKEN' was not found")?;
        let db = sled::open("my_database2").unwrap();
        (discord_token, db)
    } else {
        return Err(Error::msg("MODEが不正です").into());
    };

    // let discord_token = secret_store
    //     .get("DISCORD_TOKEN2")
    //     .context("'DISCORD_TOKEN' was not found")?;

    let sent_member_and_guild_ids = RwLock::new(HashMap::new());
    // DAO作成
    // let db = sled::open("my_database").unwrap();
    // 一旦Cloneしておく
    let own_server_repository = Arc::new(SledOwnServerRepository::new(db.clone()));
    let own_times_repository = Arc::new(SledOwnTimesRepository::new(db.clone()));
    let other_server_repository = Arc::new(SledOtherServerRepository::new(db.clone()));
    let other_times_repository = Arc::new(SledOtherTimesRepository::new(db.clone()));

    let ubiquitimes_keygenerator = Arc::new(RsaKeyGenerator::new());

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            // ここでコマンドを登録する
            // コマンド名は1~32文字じゃないとダメみたい
            // どうやらスネークケースじゃないとだめのようだ
            commands: common::commands_vec(),

            // ここでprefixを設定する
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    Duration::from_secs(3600),
                ))),
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
                    // お試しで仕込んであるやつ
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

            // イベントハンドラの登録
            event_handler: |_ctx, event, _framework, _data| {
                Box::pin(async move {
                    info!(
                        "Got an event in event handler: {:?}",
                        event.snake_case_name()
                    );
                    common::event_handler(_ctx, event, _framework, _data).await
                    // event_handler(_ctx, event, _framework, _data).await
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    sent_member_and_guild_ids,
                    own_server_repository,
                    own_times_repository,
                    other_server_repository,
                    other_times_repository,
                    ubiquitimes_keygenerator,
                })
            })
        })
        .build();

    let client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
