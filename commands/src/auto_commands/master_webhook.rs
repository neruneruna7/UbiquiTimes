//

// use crate::{Result, Error, Data, SqlitePool, Context};

// use poise::serenity_prelude::guild;
// use serde::{Serialize, Deserialize};
// use sqlx::Sqlite;
// use tracing::info;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct BotComMessage {
//     pub src: String,
//     pub dst: String,
//     pub cmd: CmdKind,
//     pub ttl: usize,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub enum CmdKind {
//     MasterWebhookAutoRegister,
//     MasterWebhookAutoRegisterResponse,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct MessageMasterWebhookAutoRegister {
//     a_server_name: String,
//     a_guild_id: u64,
//     a_master_webhook_url: String,
// }
// #[derive(Debug, Serialize, Deserialize)]
// struct MessageMasterWebhookAutoRegisterResponse {
//     b_server_name: String,
//     b_guild_id: u64,
//     b_master_channel_id: u64,
// }

// // イベントハンドラ，Readyに登録する
// pub async fn server_name_db_init(connection: &SqlitePool, ctx: Context<'_>) -> Result<()> {
//     let guild_id = ctx.guild_id();
//     let server_name = ctx.guild();

//     let guild_id = match guild_id {
//         Some(guild_id) => guild_id.0.to_string(),
//         None => {
//             ctx.say("ギルドIDを取得できませんでした．権限設定を見直してください").await?;
//             return Err(anyhow::anyhow!("GUild id can't get.  Please review your permission settings"))
//         },
//     };

//     let server_name = match server_name {
//         Some(server_name) => server_name.name,
//         None => {
//             ctx.say("サーバ名を取得できませんでした．権限設定を見直してください").await?;
//             return Err(anyhow::anyhow!("Server name can't get.  Please review your permission settings"))
//         },
//     };

//     info!("server_name: {}, guild_id: {:?}", &server_name, guild_id);

//     upsert_server_data(connection, server_name, guild_id).await?;
//     Ok(())
// }

// pub async fn upsert_server_data(connection: &SqlitePool, server_name: String, guild_id: String) -> Result<()> {
//         // upsert
//         sqlx::query!(
//             r#"
//             INSERT INTO server_data (guild_id, server_name)
//             VALUES (?, ?)
//             ON CONFLICT (guild_id)
//             DO UPDATE SET server_name = ?;
//             "#,
//             guild_id,
//             server_name,
//             server_name,
//         ).execute(connection).await?;

//         info!("server_name_db_init: upsert done");

//         Ok(())
// }

// async fn select_serverdata_from_guild_id(connection: &SqlitePool, guild_id: u64) -> Result<String> {
//     let guild_id = guild_id.to_string();
//     let row = sqlx::query!(
//         r#"
//         SELECT * FROM server_data WHERE guild_id = ?;
//         "#,
//         guild_id
//     )
//     .fetch_one(connection)
//     .await?;

//     let server_name = row.server_name.unwrap();
//     // let master_webhook_channel_id = row.server_master_channel_id;

//     Ok(server_name)
// }

// #[poise::command(prefix_command, track_edits, aliases("UTregMaster"), slash_command)]
// pub async fn ut_auto_masterhook_register(
//     ctx: Context<'_>,
//     #[description = "拡散先サーバのマスターwebhook URL"] master_webhook_url: String,
// ) -> Result<()> {
//     let connection = ctx.data().connection.clone();
//     let src_guild_id = ctx.guild_id().unwrap().0;

//     let src_server_name = select_serverdata_from_guild_id(connection.as_ref(), src_guild_id).await?;

//     let message_master_webhook_auto_register = MessageMasterWebhookAutoRegister {
//         a_server_name: src_server_name,
//         a_guild_id: src_guild_id,
//         a_master_webhook_url: master_webhook_url,
//     };

//     Ok(())
// }
