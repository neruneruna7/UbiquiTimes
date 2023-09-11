use crate::*;

// 拡散可能なサーバ一覧を表示する
#[allow(non_snake_case)]
#[command]
async fn UTserverlist(ctx: &Context, msg: &Message) -> CommandResult {
    // DBから取得する
    let db = get_db(ctx).await.ok_or(anyhow!("db is None"))?;

    let row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks;
        "#
    )
    .fetch_all(db.as_ref())
    .await?;

    let mut reply = String::new();
    for row in row {
        reply.push_str(&format!("{}\n", row.server_name));
    }

    msg.reply(ctx, reply).await?;
    Ok(())
}
