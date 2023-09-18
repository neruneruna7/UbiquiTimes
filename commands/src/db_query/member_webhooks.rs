use super::*;

// メンバーwebhookの登録
pub(crate) async fn member_webhook_insert(
    connection: &SqlitePool,
    member_webhook: MemberWebhook,
) -> anyhow::Result<()> {
    let member_id = member_webhook.src_member_id.to_string();
    let channel_id = member_webhook.dst_channel_id.to_string();
    let guild_id = member_webhook.dst_guild_id.to_string();

    sqlx::query!(
        r#"
        INSERT INTO member_webhooks (b_server_name, a_member_id, b_guild_id, b_channel_id, b_webhook_url)
        VALUES(?, ?, ?, ?, ?);
        "#,
        member_webhook.dst_server_name,
        member_id,
        guild_id,
        channel_id,
        member_webhook.dst_webhook_url
    )
    .execute(connection)
    .await?;

    Ok(())
}

// メンバーwebhookの取得
pub(crate) async fn member_webhook_select(
    connection: &SqlitePool,
    server_name: &str,
    member_id: u64,
) -> Result<MemberWebhook> {
    let member_id = member_id.to_string();
    let row = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks WHERE b_server_name = ? AND a_member_id = ?;
        "#,
        server_name,
        member_id,
    )
    .fetch_one(connection)
    .await?;

    let member_webhook = MemberWebhook::from_row(
        Some(row.id),
        &row.a_member_id,
        &row.b_server_name,
        &row.b_guild_id,
        &row.b_channel_id,
        &row.b_webhook_url,
    )?;

    Ok(member_webhook)
}

// メンバーidと一致するメンバーwebhookの全取得
//
pub(crate) async fn member_webhook_select_from_member_id(
    connection: &SqlitePool,
    member_id: u64,
) -> Result<Vec<MemberWebhook>> {
    let member_id = member_id.to_string();
    let rows = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks WHERE a_member_id = ?;
        "#,
        member_id,
    )
    .fetch_all(connection)
    .await?;

    let mut member_webhook_list = Vec::new();
    for row in rows {
        let member_webhook = MemberWebhook::from_row(
            Some(row.id),
            &row.a_member_id,
            &row.b_server_name,
            &row.b_guild_id,
            &row.b_channel_id,
            &row.b_webhook_url,
        )?;
        member_webhook_list.push(member_webhook);
    }

    Ok(member_webhook_list)
}

pub(crate) async fn member_webhook_select_all(
    connection: &SqlitePool,
) -> Result<Vec<MemberWebhook>> {
    let rows = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks;
        "#
    )
    .fetch_all(connection)
    .await?;

    let mut member_webhook_list = Vec::new();
    for row in rows {
        let member_webhook = MemberWebhook::from_row(
            Some(row.id),
            &row.a_member_id,
            &row.b_server_name,
            &row.b_guild_id,
            &row.b_channel_id,
            &row.b_webhook_url,
        )?;
        member_webhook_list.push(member_webhook);
    }

    Ok(member_webhook_list)
}

// servername, member_idを指定してメンバーwebhookを削除する
pub(crate) async fn member_webhook_delete(
    connection: &SqlitePool,
    server_name: &str,
    member_id: u64,
) -> Result<()> {
    let member_id = member_id.to_string();
    sqlx::query!(
        r#"
        DELETE FROM member_webhooks WHERE b_server_name = ? AND a_member_id = ?;
        "#,
        server_name,
        member_id
    )
    .execute(connection)
    .await?;

    Ok(())
}
