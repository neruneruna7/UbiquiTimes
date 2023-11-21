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