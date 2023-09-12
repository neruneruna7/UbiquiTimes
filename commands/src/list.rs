use poise::serenity_prelude::connection;

use crate::*;

/// 拡散可能なサーバ一覧を表示する
#[allow(non_snake_case)]
#[poise::command(prefix_command, track_edits, slash_command)]
async fn UTserverlist(
    ctx: Context<'_>) -> Result<()> {
    // DBから取得する
    let connection = ctx.data().connection.clone();

    let row = sqlx::query!(
        r#"
        SELECT * FROM master_webhooks;
        "#
    )
    .fetch_all(connection.as_ref())
    .await?;

    let mut response = String::new();
    for row in row {
        response.push_str(&format!("{}\n", row.server_name));
    }

    ctx.say(response).await?;
    Ok(())
}
