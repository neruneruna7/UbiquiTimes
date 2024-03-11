use crate::models::{OtherServer, OtherTimes, OwnServer, OwnTimes};

pub trait OwnServerRepository {
    type Error;
    // 結局のところ，複数レコードを扱うことはない
    // つねに1つのレコードしかない
    async fn upsert(&self, own_server: OwnServer) -> Result<OwnServer, Self::Error>;
    // レコードがない場合，今回はNoneとせずにエラーとする
    async fn get(&self) -> Result<OwnServer, Self::Error>;
    async fn delete(&self) -> Result<OwnServer, Self::Error>;
}

pub trait OwnTimesRepository {
    type Error;
    async fn upsert(&self, own_times: OwnTimes) -> Result<OwnTimes, Self::Error>;
    async fn get(&self, member_id: u64) -> Result<Option<OwnTimes>, Self::Error>;
    async fn get_all(&self) -> Result<Vec<OwnTimes>, Self::Error>;
    async fn delete(&self, member_id: u64) -> Result<Option<OwnTimes>, Self::Error>;
}


pub trait OtherServerRepository {
    // 正直asyncはいらない
    type Error;

    async fn upsert(&self, other_server: OtherServer) -> Result<OtherServer, Self::Error>;
    async fn get(&self, server_name: &str) -> Result<Option<OtherServer>, Self::Error>;
    async fn get_all(&self) -> Result<Vec<OtherServer>, Self::Error>;
    async fn get_from_guild_id(&self, guild_id: u64) -> Result<Option<OtherServer>, Self::Error>;
    async fn delete(&self, server_name: &str) -> Result<OtherServer, Self::Error>;
}

pub trait OtherTimesRepository {
    type Error;

    async fn upsert(&self, other_times: OtherTimes) -> Result<OtherTimes, Self::Error>;
    async fn get(&self, server_name: &str, member_id: u64) -> Result<Option<OtherTimes>, Self::Error>;
    async fn get_all(&self) -> Result<Vec<OtherTimes>, Self::Error>;
    async fn get_from_member_id(&self, member_id: u64) -> Result<Vec<OtherTimes>, Self::Error>;
    async fn delete(
        &self,
        server_name: &str,
        member_id: u64,
    ) -> Result<Option<OtherTimes>, Self::Error>;
}


