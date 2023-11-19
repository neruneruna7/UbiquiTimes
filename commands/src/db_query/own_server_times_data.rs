use super::*;
use crate::own_server::OwnTimesData;

// sqliteにmember自身のtimes情報をupsertする
pub(crate) async fn upsert_own_times_data(
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
pub(crate) async fn select_own_times_data(
    connection: &SqlitePool,
    member_id: u64,
) -> Result<OwnTimesData> {
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

    let member_times = OwnTimesData::from_row(
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

pub(crate) async fn delete_own_times_data(data: &Data, member_id: u64) -> anyhow::Result<()> {
    let connection = data.connection.clone();
    let member_id = member_id.to_string();

    sqlx::query!(
        r#"
        DELETE FROM a_member_times_data
        WHERE member_id = ?
        "#,
        member_id,
    )
    .execute(connection.as_ref())
    .await?;

    Ok(())
}

pub(crate) async fn select_own_times_data_all(data: &Data) -> anyhow::Result<Vec<OwnTimesData>> {
    let connection = data.connection.clone();

    let rows = sqlx::query!(
        r#"
        SELECT * FROM a_member_times_data
        "#,
    )
    .fetch_all(connection.as_ref())
    .await?;

    let mut own_times_data = Vec::new();

    for row in rows {
        own_times_data.push(OwnTimesData {
            member_id: row.member_id.parse::<u64>()?,
            member_name: row.member_name,
            channel_id: row.channel_id.parse::<u64>()?,
            webhook_url: row.webhook_url,
        });
    }

    Ok(own_times_data)
}
