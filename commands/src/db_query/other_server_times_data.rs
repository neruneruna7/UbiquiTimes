use super::*;

use crate::other_server::{OtherTimesData, OtherTimesDataTable};

use sled::Db;
use anyhow::{Result};
use uuid::Uuid;
use tracing::error;

pub struct OtherTimesDataTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OtherTimesDataTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OtherTimesDataTable<'a> {
    const TABLE_NAME: &'static str = "OtherTimesDataTable";

    type SledKey = String;

    type SledValue = OtherTimesDataKv;

    fn get_db(&self) -> &sled::Db {
        self.db
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
struct OtherTimesDataKv {
    other_times_data: OtherTimesData,
    key: String,
}

impl OtherTimesDataKv {
    fn new(other_times_data: OtherTimesData, key: String) -> Self { Self { other_times_data, key } }
}


// member_idとserver_nameがあって一意に定まるので，効率は悪いがuuidをキーとしておく
impl OtherTimesData{
    pub fn db_upsert(&self, db: &Db) -> Result<()> {
        let server_name = self.dst_server_name;
        let member_id = self.src_member_id;
        let data = Self::fillted_data(db, &server_name, member_id)?;
        let key = match data.len() {
            0 => Uuid::new_v4().to_string(),
            1 => data[0].key,
            _ => {
                error!("kvに永続化されたOtherTimesDataに異常があります. server_name, member_idの2つでユニークのはずですが，2つ以上データがあります");
                data[0].key
            },
        };
        let kv_value = OtherTimesDataKv::new(self.clone(), key.clone());
        let other_times_table = OtherTimesDataTable::new(db);
        let _ = other_times_table.upsert(&key, &kv_value)?;
        Ok(())
    }

    pub fn db_read_from_member_id(db: &Db, member_id: u64) -> Result<Vec<Self>> {
        let fillter_data =  Self::fillter_member_id(db, member_id)?;

        // uuid情報を削除する
        let data = fillter_data.into_iter().map(|x| x.other_times_data).collect();
        Ok(data)
    }

    pub fn db_read(db: &Db, server_name: &str, member_id: u64) -> Result<Vec<Self>> {
        let fillter_data = Self::fillted_data(db, server_name, member_id)?;

        // uuid情報を削除
        let data = fillter_data.into_iter().map(|x| x.other_times_data).collect();

        Ok(data)
    }

    pub fn db_read_all(db: &Db) -> Result<Vec<Self>> {
        let other_times_table = OtherTimesDataTable::new(db);
        let data = other_times_table.read_all()?;
        let data = data.into_iter().map(|x| x.other_times_data).collect();
        Ok(data)
    }

    pub fn db_delete(db: &Db,  server_name: &str, member_id: u64,) -> Result<()> {
        let other_times_table = OtherTimesDataTable::new(db);

        let data = Self::fillted_data(db, server_name, member_id)?;
        // 1つしか存在しないはずである
        if data.len() != 1 {
            error!("kvに永続化されたOtherTimesDataに異常があります. server_name, member_idの2つでユニークのはずですが，2つ以上データがあります")
        }
        
        let uuid = data[0].key;
        other_times_table.delete(&uuid)?;
        Ok(())
    }

    fn fillter_member_id(db: &Db, member_id: u64) -> Result<Vec<OtherTimesDataKv>> {
        let other_times_table = OtherTimesDataTable::new(db);
        let data = other_times_table.read_all()?;
        // member_idが一致するものを抽出
        let fillter_data: Vec<OtherTimesDataKv> = data.into_iter().filter(|x| x.other_times_data.src_member_id == member_id).collect();

        Ok(fillter_data)
    }

    fn fillted_data(db: &Db, server_name: &str, member_id: u64) -> Result<Vec<OtherTimesDataKv>> {
        let other_times_table = OtherTimesDataTable::new(db);
        let data = other_times_table.read_all()?;
        // member_idが一致するものを抽出
        let fillter_data: Vec<OtherTimesDataKv> = data.into_iter().filter(|x| x.other_times_data.src_member_id == member_id && x.other_times_data.dst_server_name == server_name).collect();
        Ok(fillter_data)
    }



}

// メンバーwebhookの登録
// upsert
pub(crate) async fn member_webhook_upsert(
    connection: &SqlitePool,
    member_webhook: OtherTimesData,
) -> anyhow::Result<()> {
    let member_id = member_webhook.src_member_id.to_string();
    let channel_id = member_webhook.dst_channel_id.to_string();
    let guild_id = member_webhook.dst_guild_id.to_string();

    sqlx::query!(
        r#"
        INSERT INTO member_webhooks (b_server_name, a_member_id, b_guild_id, b_channel_id, b_webhook_url)
        VALUES(?, ?, ?, ?, ?)
        ON CONFLICT(b_guild_id, a_member_id) DO UPDATE SET
            b_server_name = ?,
            b_channel_id = ?,
            b_webhook_url = ?;
        "#,
        member_webhook.dst_server_name,
        member_id,
        guild_id,
        channel_id,
        member_webhook.dst_webhook_url,
        member_webhook.dst_server_name,
        channel_id,
        member_webhook.dst_webhook_url,
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
) -> Result<OtherTimesData> {
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

    let member_webhook = OtherTimesData::from_row(
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
) -> Result<Vec<OtherTimesData>> {
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
        let member_webhook = OtherTimesData::from_row(
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
) -> Result<Vec<OtherTimesData>> {
    let rows = sqlx::query!(
        r#"
        SELECT * FROM member_webhooks;
        "#
    )
    .fetch_all(connection)
    .await?;

    let mut member_webhook_list = Vec::new();
    for row in rows {
        let member_webhook = OtherTimesData::from_row(
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
