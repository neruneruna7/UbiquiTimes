use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Error};
use commands::other_server_repository::{
    sled_other_server_repository, SledOtherServerRepository, SledOtherTimesRepository,
};
use commands::own_server_repository::{
    sled_own_server_repository, sled_own_times_repository, SledOwnServerRepository,
    SledOwnTimesRepository,
};
use commands::sign::keys_gen::RsaKeyGenerator;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::RwLock;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;

use commands::global_data::{Context, Data};
use commands::help;
use sled::Db;
use tracing::info;

// struct Data {} // User data, which is stored and accessible in all command invocations
// type Error = Box<dyn std::error::Error + Send + Sync>;
// type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with "world!"
#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("world!").await?;
    Ok(())
}

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    // #[shuttle_static_folder::StaticFolder(folder = "db")] static_folder: PathBuf,
) -> ShuttlePoise<Data, Error> {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;
    // let database_url = secret_store
    //     .get("DATABASE_URL")
    //     .context("'DATABASE_URL' was not found")?;

    let sent_member_and_guild_ids = RwLock::new(HashMap::new());
    // DAO作成
    let db = sled::open("my_database").unwrap();
    // 一旦Cloneしておく
    let own_server_repository = Arc::new(SledOwnServerRepository::new(db.clone()));
    let own_times_repository = Arc::new(SledOwnTimesRepository::new(db.clone()));
    let other_server_repository = Arc::new(SledOtherServerRepository::new(db.clone()));
    let other_times_repository = Arc::new(SledOtherTimesRepository::new(db.clone()));

    let ubiquitimes_keygenerator = Arc::new(RsaKeyGenerator::new());

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            // hello(),
            commands: common::commands_vec(),
            ..Default::default()
        })
        .token(discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
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
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
