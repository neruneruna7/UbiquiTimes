use super::{OtherServerRepository, OtherTimesRepository, OtherTimesResult};

use crate::{other_server::OtherTimes, sled_table::SledTable};
use serde::{Deserialize, Serialize};
use tracing::error;

pub struct SledOtherTimesRepository {
    db: sled::Db,
}

impl SledOtherTimesRepository {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }

    fn fillter_member_id(&self, member_id: u64) -> OtherTimesResult<Vec<OtherTimes>> {
        let other_times_table = OtherTimesTable::new(&self.db);
        let data = other_times_table.read_all()?;
        // member_idが一致するものを抽出する
        let filtered_data: Vec<OtherTimes> = data
            .into_iter()
            .filter(|x| x.other_times_data.src_member_id == member_id)
            .map(|x| x.other_times_data)
            .collect();

        Ok(filtered_data)
    }

    fn fillted_data(&self, server_name: &str, member_id: u64) -> OtherTimesResult<Vec<OtherTimes>> {
        let other_times_table = OtherTimesTable::new(&self.db);
        let data = other_times_table.read_all()?;
        // member_idが一致するものを抽出
        let fillter_data: Vec<OtherTimes> = data
            .into_iter()
            .filter(|x| {
                x.other_times_data.src_member_id == member_id
                    && x.other_times_data.dst_server_name == server_name
            })
            .map(|x| x.other_times_data)
            .collect();
        Ok(fillter_data)
    }
}

impl OtherTimesRepository for SledOtherTimesRepository {
    async fn upsert(&self, other_times: OtherTimes) -> super::OtherTimesResult<OtherTimes> {
        let other_times_table = OtherTimesTable::new(&self.db);
        let kv = Into::<OtherTimesKv>::into(other_times.clone());
        other_times_table.upsert(&kv.key, &kv)?;
        Ok(other_times)
    }

    async fn get(
        &self,
        server_name: &str,
        member_id: u64,
    ) -> super::OtherTimesResult<Option<OtherTimes>> {
        let mut fillter_data = self.fillted_data(server_name, member_id)?;

        // 一意に定まるはずなので，要素数が2以上はおかしい
        if fillter_data.len() > 1 {
            error!("server_name, member_idが一致するOtherTimesDataが2つ以上あります");
        }
        // 何もないならNoneを返す
        if fillter_data.is_empty() {
            return Ok(None);
        }
        let data = fillter_data.remove(0);
        Ok(Some(data))
    }

    async fn get_all(&self) -> super::OtherTimesResult<Vec<OtherTimes>> {
        let other_times_table = OtherTimesTable::new(&self.db);
        let data = other_times_table.read_all()?;
        let data = data.into_iter().map(|x| x.other_times_data).collect();
        Ok(data)
    }

    async fn get_from_member_id(&self, member_id: u64) -> super::OtherTimesResult<Vec<OtherTimes>> {
        let fillter_data = self.fillter_member_id(member_id)?;

        Ok(fillter_data)
    }

    async fn delete(
        &self,
        server_name: &str,
        member_id: u64,
    ) -> super::OtherTimesResult<Option<OtherTimes>> {
        let other_times_table = OtherTimesTable::new(&self.db);

        let mut data = self.fillted_data(server_name, member_id)?;
        // 2つ以上ならエラー
        if data.len() != 1 {
            error!("server_name, member_idが一致するOtherTimesDataが2つ以上あります");
        }
        // 0なら処理をしない
        if data.is_empty() {
            return Ok(None);
        }

        let other_times: OtherTimesKv = data.remove(0).into();
        other_times_table.delete(&other_times.key)?;
        Ok(Some(other_times.other_times_data))
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
/// 設計上はpubにする必要はないのだが，pubトレイトの関連型で使う都合でpubにする必要がある
pub struct OtherTimesKv {
    other_times_data: OtherTimes,
    key: String,
}

// OtherTimesとのFromトレイトを実装する
// member_idとguild_idを組み合わせると一意になるはず
impl From<OtherTimes> for OtherTimesKv {
    fn from(other_times_data: OtherTimes) -> Self {
        let key = format!(
            "{}_{}",
            other_times_data.src_member_id, other_times_data.dst_guild_id
        );
        Self {
            other_times_data,
            key,
        }
    }
}

pub struct OtherTimesTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OtherTimesTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OtherTimesTable<'a> {
    const TABLE_NAME: &'static str = "OtherTimesDataTable";

    type SledKey = String;

    type SledValue = OtherTimesKv;

    fn get_db(&self) -> &sled::Db {
        self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sled_other_times_repository_upsert_get() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let other_times_repository = SledOtherTimesRepository::new(db);
        let other_times = OtherTimes::new(1, "server_name", 2, 3, "webhook_url");
        let _result = other_times_repository
            .upsert(other_times.clone())
            .await
            .unwrap();

        let get = other_times_repository
            .get("server_name", 1)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(other_times, get);
    }

    #[tokio::test]
    async fn sled_other_times_repository_upsert_get_all() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let other_times_repository = SledOtherTimesRepository::new(db);
        let other_times1 = OtherTimes::new(1, "server_name", 2, 3, "webhook_url");
        let other_times2 = OtherTimes::new(4, "server_name2", 5, 6, "webhook_url2");
        let _result1 = other_times_repository
            .upsert(other_times1.clone())
            .await
            .unwrap();
        let _result2 = other_times_repository
            .upsert(other_times2.clone())
            .await
            .unwrap();

        let get = other_times_repository.get_all().await.unwrap();
        assert_eq!(vec![other_times1, other_times2], get);
    }
}
