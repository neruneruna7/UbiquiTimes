use super::{OwnTimes, OwnTimesRepository, OwnTimesResult};
use crate::sled_table::SledTable;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct SledOwnTimesRepository {
    db: sled::Db,
}

impl SledOwnTimesRepository {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }
}

impl OwnTimesRepository for SledOwnTimesRepository {
    async fn upsert(&self, own_times: OwnTimes) -> OwnTimesResult<OwnTimes> {
        let own_times_table = OwnTimesTable::new(&self.db);
        let kv = Into::<OwnTimesKv>::into(own_times.clone());
        own_times_table.upsert(&kv.key, &kv)?;
        Ok(own_times)
    }

    async fn get(&self, member_id: u64) -> OwnTimesResult<Option<OwnTimes>> {
        let own_times_table = OwnTimesTable::new(&self.db);
        let key = format!("{}", member_id);
        let data = own_times_table.read(&key)?;
        // Option型の内部の型を変えたい
        let data = data.map(|x| x.own_times_data);

        Ok(data)
    }

    async fn get_all(&self) -> OwnTimesResult<Vec<OwnTimes>> {
        let own_times_table = OwnTimesTable::new(&self.db);
        let data = own_times_table.read_all()?;
        let data = data.into_iter().map(|x| x.own_times_data).collect();
        Ok(data)
    }

    async fn delete(&self, member_id: u64) -> OwnTimesResult<Option<OwnTimes>> {
        let own_times_table = OwnTimesTable::new(&self.db);
        let key = format!("{}", member_id);
        let data = own_times_table.read(&key)?;
        own_times_table.delete(&key)?;

        let data = data.map(|x| x.own_times_data);
        Ok(data)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct OwnTimesKv {
    own_times_data: OwnTimes,
    key: String,
}

impl From<OwnTimes> for OwnTimesKv {
    fn from(own_times_data: OwnTimes) -> Self {
        // 自分のサーバに自分のTimesは１つしかないはず
        // なので，member_idさえあれば一意になるはず
        let key = format!("{}", own_times_data.member_id);
        Self {
            own_times_data,
            key,
        }
    }
}

pub struct OwnTimesTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OwnTimesTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OwnTimesTable<'a> {
    const TABLE_NAME: &'static str = "OwnTimesDataTable";

    type SledKey = String;

    type SledValue = OwnTimesKv;

    fn get_db(&self) -> &sled::Db {
        self.db
    }
}
