use sqlx::Row;
use tracing::debug;

use super::*;

// sqliteにmember自身のtimes情報をupsertする
pub(crate) async fn upsert_member_times(
    connection: &SqlitePool,
    member_id: u64,
    member_name: &str,
    channel_id: u64,
    webhook_url: &str,
) -> Result<()> {
    // let mut conn = pool.acquire().await?;
    let member_id = member_id.to_string();
    let channel_id = channel_id.to_string();
    let _a = sqlx::query!(
        r#"
        INSERT INTO a_member_times_data (member_id, member_name, channel_id, webhook_url)
        VALUES (?, ?, ?, ?)
        ON CONFLICT (member_id) DO UPDATE SET member_name = ?, channel_id = ?, webhook_url = ?
        "#,
        member_id,
        member_name,
        channel_id,
        webhook_url,
        member_name,
        channel_id,
        webhook_url,
    )
    .execute(connection)
    .await?;

    info!("{:?}", _a);

    Ok(())
}

// 取得する
pub(crate) async fn select_member_times(
    connection: &SqlitePool,
    member_id: u64,
) -> Result<MemberTimesData> {
    let member_id = member_id.to_string();
    let member_times_row = sqlx::query!(
        r#"
        SELECT * FROM a_member_times_data
        WHERE member_id = ?;
        "#,
        member_id,
    )
    .fetch_one(connection)
    .await?;

    let member_times = MemberTimesData::from_row(
        &member_times_row.member_id,
        &member_times_row.member_name,
        &member_times_row.channel_id,
        &member_times_row.webhook_url,
    )?;

    Ok(member_times)
}

// 引数としてとったmember_idが存在していればtrueを返す
pub(crate) async fn is_exist_member_times(connection: &SqlitePool, member_id: u64) -> Result<bool> {
    let member_id = member_id.to_string();
    let member_times_row = sqlx::query!(
        r#"
        SELECT * FROM a_member_times_data
        WHERE member_id = ?;
        "#,
        member_id,
    )
    .fetch_optional(connection)
    .await?;

    Ok(member_times_row.is_some())
}
