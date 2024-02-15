use super::{OwnServer, OwnServerRepository, OwnServerResult};
use crate::sled_table::SledTable;
use anyhow::Result;
use tracing::error;

pub struct SledOwnServerRepository {
    db: sled::Db,
}

impl SledOwnServerRepository {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }
}

impl OwnServerRepository for SledOwnServerRepository {
    // keyを引数にとる必要はない
    // 内部で読んでるseldtableトレイトの都合で引数を取る関数を使っているが，
    // それは適当に空の文字列をわたしておく
    async fn upsert(&self, own_server: OwnServer) -> OwnServerResult<OwnServer> {
        let own_server_table = OwnServerTable::new(&self.db);
        own_server_table.upsert(&own_server.server_name, &own_server)?;
        Ok(own_server)
    }

    async fn get(&self) -> OwnServerResult<OwnServer> {
        let own_server_table = OwnServerTable::new(&self.db);
        let data = own_server_table.read(&String::new())?;
        // NoneのときはErrを返す
        match data {
            Some(data) => Ok(data),
            None => Err(anyhow::anyhow!(
                "OwnServerデータがありません. 先にupsert関数を呼んで登録してください"
            )
            .into()),
        }
    }

    async fn delete(&self) -> OwnServerResult<OwnServer> {
        let own_server_table = OwnServerTable::new(&self.db);
        let data = own_server_table.read(&String::new())?;
        own_server_table.delete(&String::new())?;

        // NoneのときはErrを返す
        match data {
            Some(data) => Ok(data),
            None => Err(
                anyhow::anyhow!("OwnServerデータがありません. 削除するデータがありません").into(),
            ),
        }
    }
}

struct OwnServerTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OwnServerTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

const KEY_NAME: &'static str = "OwnServer";
impl<'a> SledTable for OwnServerTable<'a> {
    const TABLE_NAME: &'static str = "OwnServerDataTable";
    // 自身のサーバーデータは常に1つしかない
    // keyにどんな値を渡したとしても同じキーを使うように
    // そのためにトレイトのデフォルト実装を書き換えている

    type SledKey = String;

    type SledValue = OwnServer;

    fn get_db(&self) -> &sled::Db {
        self.db
    }

    fn upsert(&self, _key: &Self::SledKey, value: &Self::SledValue) -> Result<()> {
        let key = KEY_NAME;
        let value = serde_json::to_string(value)?;
        let byte_key = value.as_bytes();
        let db = self.get_db();
        db.open_tree(Self::TABLE_NAME)?.insert(key, byte_key)?;
        Ok(())
    }

    fn read(&self, _key: &Self::SledKey) -> Result<Option<Self::SledValue>> {
        let db = self.get_db();
        let byte_key = KEY_NAME;
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

    fn delete(&self, _key: &Self::SledKey) -> Result<()> {
        let db = self.get_db();
        let byte_key = KEY_NAME;
        db.open_tree(Self::TABLE_NAME)?.remove(byte_key)?;
        Ok(())
    }
}
