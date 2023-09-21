use super::*;
use types::own_server_data::ServerData;

/// 自身のマスターwebhookを a_server_data テーブルにupsertする
pub(crate) async fn upsert_own_server_data(
    connection: &SqlitePool,
    server_name: &str,
    guild_id: &str,
    master_channel_id: &str,
    master_webhook_url: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO a_server_data (server_name, guild_id, master_channel_id, master_webhook_url)
        VALUES(?, ?, ?, ?)
        ON CONFLICT(guild_id) DO UPDATE SET server_name = ?, master_channel_id = ?, master_webhook_url = ?;
        "#,
        server_name,
        guild_id,
        master_channel_id,
        master_webhook_url,
        server_name,
        master_channel_id,
        master_webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}

// 取得する
pub(crate) async fn select_own_server_data(
    connection: &SqlitePool,
    guild_id: u64,
) -> anyhow::Result<ServerData> {
    let guild_id = guild_id.to_string();
    let row = sqlx::query!(
        r#"
        SELECT * FROM a_server_data WHERE guild_id = ?;
        "#,
        guild_id
    )
    .fetch_one(connection)
    .await?;

    let a_server_data = ServerData::from_row(
        &row.guild_id.to_string(),
        &row.server_name,
        &row.master_channel_id.to_string(),
        &row.master_webhook_url,
    )?;

    Ok(a_server_data)
}

// 取得 guild_idを用いない
// このデータは1つしかないことを前提にしている
pub(crate) async fn select_own_server_data_without_guild_id(
    connection: &SqlitePool,
) -> anyhow::Result<ServerData> {
    let row = sqlx::query!(
        r#"
        SELECT * FROM a_server_data;
        "#
    )
    .fetch_one(connection)
    .await?;

    let a_server_data = ServerData::from_row(
        &row.guild_id.to_string(),
        &row.server_name,
        &row.master_channel_id.to_string(),
        &row.master_webhook_url,
    )?;

    Ok(a_server_data)
}