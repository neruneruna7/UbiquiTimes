use anyhow::{Error, Result};

use poise::serenity_prelude::{self as serenity};

use serenity::{model::channel::Message, webhook::Webhook};

use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc, Mutex},
};

pub mod master_webhook;
pub mod member_webhook;

mod db_query;

// Types used by all command functions
// すべてのコマンド関数で使用される型
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
pub struct Data {
    pub votes: Mutex<HashMap<String, u32>>,
    pub poise_mentions: AtomicU32,
    pub connection: Arc<SqlitePool>,
}

async fn create_webhook_from_channel(
    ctx: Context<'_>,
    msg: &Message,
    name: &str,
) -> anyhow::Result<Webhook> {
    let webhook = msg.channel_id.create_webhook(ctx, name).await?;
    Ok(webhook)
}

// 相手サーバーに対して１つだけ存在するwebhook
#[derive(Debug)]
pub struct MasterWebhook {
    pub id: Option<u64>,
    pub server_name: String,
    pub guild_id: u64,
    pub webhook_url: String,
}

impl MasterWebhook {
    fn from(_id: Option<i64>, server_name: &str, guild_id: u64, webhook_url: &str) -> Self {
        Self {
            id: None,
            server_name: server_name.to_string(),
            guild_id,
            webhook_url: webhook_url.to_string(),
        }
    }

    fn from_row(
        _id: Option<i64>,
        server_name: &str,
        guild_id: &str,
        webhook_url: &str,
    ) -> Result<Self> {
        let guild_id = guild_id.parse::<u64>()?;
        Ok(Self {
            id: None,
            server_name: server_name.to_string(),
            guild_id,
            webhook_url: webhook_url.to_string(),
        })
    }
}

#[derive(Debug)]
// 個々人が持つwebhook
pub struct MemberWebhook {
    pub src_member_id: u64,
    pub dst_server_name: String,
    pub dst_guild_id: u64,
    pub dst_channel_id: u64,
    pub dst_webhook_url: String,
}

impl MemberWebhook {
    fn from(
        src_member_id: u64,
        dst_server_name: &str,
        dst_guild_id: u64,
        dst_channel_id: u64,
        dst_webhook_url: &str,
    ) -> Self {
        Self {
            src_member_id,
            dst_server_name: dst_server_name.to_string(),
            dst_guild_id,
            dst_channel_id,
            dst_webhook_url: dst_webhook_url.to_string(),
        }
    }

    fn from_row(
        src_member_id: &str,
        dst_server_name: &str,
        dst_guild_id: &str,
        dst_channel_id: &str,
        dst_webhook_url: &str,
    ) -> Result<Self> {
        Ok(Self {
            src_member_id: src_member_id.parse()?,
            dst_server_name: dst_server_name.to_string(),
            dst_guild_id: dst_guild_id.parse()?,
            dst_channel_id: dst_channel_id.parse()?,
            dst_webhook_url: dst_webhook_url.to_string(),
        })
    }
}

#[derive(Debug)]
struct MemberTimesData {
    member_id: u64,
    member_name: String,
    channel_id: u64,
    webhook_url: String,
}

impl MemberTimesData {
    fn from(member_id: u64, member_name: &str, channel_id: u64, webhook_url: &str) -> Self {
        Self {
            member_id,
            member_name: member_name.to_string(),
            channel_id,
            webhook_url: webhook_url.to_string(),
        }
    }

    fn from_row(
        member_id: &str,
        member_name: &str,
        channel_id: &str,
        webhook_url: &str,
    ) -> Result<Self> {
        Ok(Self {
            member_id: member_id.parse::<u64>()?,
            member_name: member_name.to_string(),
            channel_id: channel_id.parse::<u64>()?,
            webhook_url: webhook_url.to_string(),
        })
    }
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

struct AServerData {
    pub guild_id: u64,
    pub server_name: String,
    pub master_channel_id: u64,
    pub master_webhook_url: String,
}

impl AServerData {
    fn from(
        guild_id: u64,
        server_name: &str,
        master_channel_id: u64,
        master_webhook_url: &str,
    ) -> Self {
        Self {
            guild_id,
            server_name: server_name.to_string(),
            master_channel_id,
            master_webhook_url: master_webhook_url.to_string(),
        }
    }

    fn from_row(
        guild_id: &str,
        server_name: &str,
        master_channel_id: &str,
        master_webhook_url: &str,
    ) -> anyhow::Result<Self> {
        let guild_id = guild_id.parse::<u64>()?;
        let master_channel_id = master_channel_id.parse::<u64>()?;

        Ok(Self {
            guild_id,
            server_name: server_name.to_string(),
            master_channel_id,
            master_webhook_url: master_webhook_url.to_string(),
        })
    }
}

// /// Vote for something
// ///
// /// Enter `~vote pumpkin` to vote for pumpkins
// #[poise::command(prefix_command, slash_command)]
// pub async fn vote(
//     ctx: Context<'_>,
//     #[description = "What to vote for"] choice: String,
// ) -> Result<()> {
//     // Lock the Mutex in a block {} so the Mutex isn't locked across an await point
//     let num_votes = {
//         let mut hash_map = ctx.data().votes.lock().unwrap();
//         let num_votes = hash_map.entry(choice.clone()).or_default();
//         *num_votes += 1;
//         *num_votes
//     };

//     let response = format!("Successfully voted for {choice}. {choice} now has {num_votes} votes!");
//     ctx.say(response).await?;
//     Ok(())
// }

// /// Retrieve number of votes
// ///
// /// Retrieve the number of votes either in general, or for a specific choice:
// /// ```
// /// ~getvotes
// /// ~getvotes pumpkin
// /// ```
// #[poise::command(prefix_command, track_edits, aliases("votes"), slash_command)]
// pub async fn getvotes(
//     ctx: Context<'_>,
//     #[description = "Choice to retrieve votes for"] choice: Option<String>,
// ) -> Result<()> {
//     if let Some(choice) = choice {
//         let num_votes = *ctx.data().votes.lock().unwrap().get(&choice).unwrap_or(&0);
//         let response = match num_votes {
//             0 => format!("Nobody has voted for {} yet", choice),
//             _ => format!("{} people have voted for {}", num_votes, choice),
//         };
//         ctx.say(response).await?;
//     } else {
//         let mut response = String::new();
//         for (choice, num_votes) in ctx.data().votes.lock().unwrap().iter() {
//             response += &format!("{}: {} votes", choice, num_votes);
//         }

//         if response.is_empty() {
//             response += "Nobody has voted for anything yet :(";
//         }

//         ctx.say(response).await?;
//     };

//     Ok(())
// }
