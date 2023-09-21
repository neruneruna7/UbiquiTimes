use std::sync::Arc;

use anyhow::{Context as _, Error};
use poise::serenity_prelude as serenity;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;

use sqlx::SqlitePool;

use commands::member_webhook::auto::{
    ut_times_set, ut_times_show, ut_times_ubiqui_setting_send, ut_times_unset,
};
use commands::member_webhook::manual::{
    ut_delete, ut_list, ut_member_webhook_reg_manual, ut_times_release,
};
use commands::{
    master_webhook::manual::{
        ut_get_master_hook, ut_serverlist, ut_set_other_masterhook, ut_set_own_masterhook,
    },
    member_webhook::auto,
};

use commands::help;
use commands::types::global_data::{Context, Data};
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
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;
    let database_url = secret_store
        .get("DATABASE_URL")
        .context("'DATABASE_URL' was not found")?;

    let pool = SqlitePool::connect(&database_url).await.unwrap();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                hello(),
                help(),
                ut_set_own_masterhook(),
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
            ..Default::default()
        })
        .token(discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    connection: Arc::new(pool),
                })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
