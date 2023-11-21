use super::*;

use crate::other_server::OtherServerData;


pub async fn master_webhook_upsert(
    connection: &SqlitePool,
    master_webhook: &OtherServerData,
) -> anyhow::Result<()> {
    let guild_id = master_webhook.guild_id.to_string();

    sqlx::query!(
        r#"
        INSERT INTO master_webhooks (server_name, guild_id, webhook_url, public_key_pem)
        VALUES(?, ?, ?, ?)
        ON CONFLICT(guild_id) DO UPDATE SET server_name = ?, webhook_url = ?, public_key_pem = ?
        ;
        "#,
        master_webhook.server_name,
        guild_id,
        master_webhook.webhook_url,
        master_webhook.public_key_pem,
        master_webhook.server_name,
        master_webhook.webhook_url,
        master_webhook.public_key_pem,
    )
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn master_webhook_select_from_servername(
    connection: &SqlitePool,
    server_name: &str,
) -> anyhow::Result<OtherServerData> {
    let row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks WHERE server_name = ?;
        "#,
        server_name
    )
    .fetch_one(connection)
    .await?;

    OtherServerData::from_row(
        &row.guild_id,
        &row.server_name,
        &row.webhook_url,
        &row.public_key_pem,
    )
}

// guild_idを指定して取得する
pub async fn master_webhook_select_from_guild_id(
    connection: &SqlitePool,
    guild_id: u64,
) -> anyhow::Result<OtherServerData> {
    let guild_id = guild_id.to_string();
    let row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks WHERE guild_id = ?;
        "#,
        guild_id
    )
    .fetch_one(connection)
    .await?;

    OtherServerData::from_row(
        &row.guild_id,
        &row.server_name,
        &row.webhook_url,
        &row.public_key_pem,
    )
}

// すべてのマスターwebhookを取得する
// 複数の行がとれるので、Vecに格納して返す
pub async fn master_webhook_select_all(
    connection: &SqlitePool,
) -> anyhow::Result<Vec<OtherServerData>> {
    let rows = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks;
        "#,
    )
    .fetch_all(connection)
    .await?;

    let mut master_webhooks = Vec::new();

    for row in rows {
        let master_webhook = OtherServerData::from_row(
            &row.guild_id,
            &row.server_name,
            &row.webhook_url,
            &row.public_key_pem,
        )?;
        master_webhooks.push(master_webhook);
    }

    Ok(master_webhooks)
}

pub async fn master_webhook_delete(
    connection: &SqlitePool,
    server_name: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM master_webhooks WHERE server_name = ?;
        "#,
        server_name
    )
    .execute(connection)
    .await?;

    Ok(())
}
