use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Error};
use poise::serenity_prelude as serenity;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;

use commands::global_data::{Context, Data};
use commands::help;
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
                Ok(
                //     Data {
                //     connection: Arc::new(pool),
                // }
            )
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
