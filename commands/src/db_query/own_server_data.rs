use super::*;
use types::own_server_data::ServerData;

/// 自身のマスターwebhookを a_server_data テーブルにupsertする
pub(crate) async fn upsert_own_server_data(
    connection: &SqlitePool,
    server_data: &ServerData,
    // server_name: &str,
    // guild_id: &str,
    // master_channel_id: &str,
    // master_webhook_url: &str,
    // private_key_pem: &str,
    // public_key_pem: &str,
) -> anyhow::Result<()> {
    let guild_id = server_data.guild_id.to_string();
    let master_channel_id = server_data.master_channel_id.to_string();
    sqlx::query!(
        r#"
        INSERT INTO a_server_data (server_name, guild_id, master_channel_id, master_webhook_url, private_key_pem, public_key_pem)
        VALUES(?, ?, ?, ?, ?, ?)
        ON CONFLICT(guild_id) DO UPDATE SET server_name = ?, master_channel_id = ?, master_webhook_url = ?, private_key_pem = ?, public_key_pem = ?;
        "#,
        server_data.server_name,
        guild_id,
        master_channel_id,
        server_data.master_webhook_url,
        server_data.private_key_pem,
        server_data.public_key_pem,
        server_data.server_name,
        master_channel_id,
        server_data.master_webhook_url,
        server_data.private_key_pem,
        server_data.public_key_pem,
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
        &row.guild_id,
        &row.server_name,
        &row.master_channel_id.to_string(),
        &row.master_webhook_url,
        &row.private_key_pem,
        &row.public_key_pem,
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
        &row.guild_id,
        &row.server_name,
        &row.master_channel_id.to_string(),
        &row.master_webhook_url,
        &row.private_key_pem,
        &row.public_key_pem,
    )?;

    Ok(a_server_data)
}
