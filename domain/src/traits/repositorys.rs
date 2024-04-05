use crate::models::guild_data::{OtherGuild, OtherTimes, OwnGuild, OwnTimes};

// type Result<T>;
// で関連型を使うのがうまくいくのかどうか調べるため，ここだけそうしている
pub trait OwnGuildRepository {
    type Result<T>;
    // 結局のところ，複数レコードを扱うことはない
    // つねに1つのレコードしかない
    fn upsert(
        &self,
        own_server: OwnGuild,
    ) -> impl std::future::Future<Output = Self::Result<OwnGuild>> + Send;
    // レコードがない場合，今回はNoneとせずにエラーとする
    fn get(&self) -> impl std::future::Future<Output = Self::Result<OwnGuild>> + Send;
    fn delete(&self) -> impl std::future::Future<Output = Self::Result<OwnGuild>> + Send;
}

pub trait OwnTimesRepository {
    type Result<T>;
    fn upsert(
        &self,
        own_times: OwnTimes,
    ) -> impl std::future::Future<Output = Self::Result<OwnTimes>> + Send;
    fn get(
        &self,
        member_id: u64,
    ) -> impl std::future::Future<Output = Self::Result<Option<OwnTimes>>> + Send;
    fn get_all(&self) -> impl std::future::Future<Output = Self::Result<Vec<OwnTimes>>> + Send;
    fn delete(
        &self,
        member_id: u64,
    ) -> impl std::future::Future<Output = Self::Result<Option<OwnTimes>>> + Send;
}

pub trait OtherGuildRepository {
    // 正直asyncはいらない
    type Result<T>;

    fn upsert(
        &self,
        other_server: OtherGuild,
    ) -> impl std::future::Future<Output = Self::Result<OtherGuild>> + Send;
    fn get(
        &self,
        server_name: &str,
    ) -> impl std::future::Future<Output = Self::Result<Option<OtherGuild>>> + Send;
    fn get_all(&self) -> impl std::future::Future<Output = Self::Result<Vec<OtherGuild>>> + Send;
    fn get_from_guild_id(
        &self,
        guild_id: u64,
    ) -> impl std::future::Future<Output = Self::Result<Option<OtherGuild>>> + Send;
    fn delete(
        &self,
        server_name: &str,
    ) -> impl std::future::Future<Output = Self::Result<OtherGuild>> + Send;
}

pub trait OtherTimesRepository {
    type Result<T>;

    fn upsert(
        &self,
        other_times: OtherTimes,
    ) -> impl std::future::Future<Output = Self::Result<OtherTimes>> + Send;
    fn get(
        &self,
        server_name: &str,
        member_id: u64,
    ) -> impl std::future::Future<Output = Self::Result<Option<OtherTimes>>> + Send;
    fn get_all(&self) -> impl std::future::Future<Output = Self::Result<Vec<OtherTimes>>> + Send;
    fn get_from_member_id(
        &self,
        member_id: u64,
    ) -> impl std::future::Future<Output = Self::Result<Vec<OtherTimes>>> + Send;
    fn delete(
        &self,
        server_name: &str,
        member_id: u64,
    ) -> impl std::future::Future<Output = Self::Result<Option<OtherTimes>>> + Send;
}
