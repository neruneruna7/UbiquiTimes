use super::*;
use crate::own_server::OwnServerData;


use anyhow::Result;
use sled::Db;

pub struct OwnServerDataTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OwnServerDataTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OwnServerDataTable<'a> {
    const TABLE_NAME: &'static str = "OwnServerDataTable";
    // const KEY_NAME: &'static str = "OwnServerData";
    // 自身のサーバーデータは常に1つしかない
    // keyにどんな値を渡したとしても同じキーを使うように
    // それぞれにkeyの引数を減らして実装したいが，その方法がわからないのでこの手段をとった
    // クソコードだね

    type SledKey = String;

    type SledValue = OwnServerData;

    fn get_db(&self) -> &sled::Db {
        self.db
    }

    fn upsert(&self, _key: &Self::SledKey, value: &Self::SledValue) -> Result<()> {
        let key = "OwnServerData";
        let value = serde_json::to_string(value)?;
        let byte_key = value.as_bytes();
        let db = self.get_db();
        db.open_tree(Self::TABLE_NAME)?.insert(key, byte_key)?;
        Ok(())
    }

    fn read(&self, _key: &Self::SledKey) -> Result<Option<Self::SledValue>> {
        let db = self.get_db();
        let byte_key = "OwnServerData";
        let ret = db.open_tree(Self::TABLE_NAME)?.get(byte_key)?;
        match ret {
            Some(ivec) => {
                let string = String::from_utf8(ivec.to_vec())?;
                let value = serde_json::from_str::<Self::SledValue>(&string)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn read_all(&self) -> Result<Vec<Self::SledValue>> {
        let db = self.get_db();
        let mut ret = Vec::new();
        let tree = db.open_tree(Self::TABLE_NAME)?;
        for item in tree.iter() {
            let (_key, value) = item?;
            let string = String::from_utf8(value.to_vec())?;
            let value = serde_json::from_str::<Self::SledValue>(&string)?;
            ret.push(value);
        }
        Ok(ret)
    }

    fn delete(&self, _key: &Self::SledKey) -> Result<()> {
        let db = self.get_db();
        let byte_key = "OwnServerData";
        db.open_tree(Self::TABLE_NAME)?.remove(byte_key)?;
        Ok(())
    }
}

impl OwnServerData {
    // 必ず1つしか存在しないデータなので，keyはなんでもいい
    // traitの方でkeyは固定している
    pub fn db_upsert(&self, db: &Db) -> Result<()> {
        let key = String::from("any key becouse no use");
        let own_server_table = OwnServerDataTable::new(db);
        own_server_table.upsert(&key, self)?;
        Ok(())
    }

    pub fn db_read(db: &Db) -> Result<Option<Self>> {
        let key = String::from("any key becouse no use");
        let own_server_table = OwnServerDataTable::new(db);
        let ret = own_server_table.read(&key)?;
        Ok(ret)
    }

    pub fn db_delete(db: &Db) -> Result<()> {
        let key = String::from("any key becouse no use");
        let own_server_table = OwnServerDataTable::new(db);
        own_server_table.delete(&key)?;
        Ok(())
    }
}
