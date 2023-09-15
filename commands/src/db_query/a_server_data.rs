use super::*;

/// 自身のマスターwebhookを a_server_data テーブルにupsertする
pub(crate) async fn upsert_a_server_data(
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
