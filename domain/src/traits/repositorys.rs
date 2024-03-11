use crate::models::guild_data::{OtherGuild, OtherTimes, OwnGuild, OwnTimes};

// type Result<T>;
// で関連型を使うのがうまくいくのかどうか調べるため，ここだけそうしている
pub trait OwnGuildRepository {
    type Result<T>;
    // 結局のところ，複数レコードを扱うことはない
    // つねに1つのレコードしかない
    async fn upsert(&self, own_server: OwnGuild) -> Self::Result<OwnGuild>;
    // レコードがない場合，今回はNoneとせずにエラーとする
    async fn get(&self) -> Self::Result<OwnGuild>;
    async fn delete(&self) -> Self::Result<OwnGuild>;
}

pub trait OwnTimesRepository {
    type Result<T>;
    async fn upsert(&self, own_times: OwnTimes) -> Self::Result<OwnTimes>;
    async fn get(&self, member_id: u64) -> Self::Result<Option<OwnTimes>>;
    async fn get_all(&self) -> Self::Result<Vec<OwnTimes>>;
    async fn delete(&self, member_id: u64) -> Self::Result<Option<OwnTimes>>;
}

pub trait OtherGuildRepository {
    // 正直asyncはいらない
    type Result<T>;

    async fn upsert(&self, other_server: OtherGuild) -> Self::Result<OtherGuild>;
    async fn get(&self, server_name: &str) -> Self::Result<Option<OtherGuild>>;
    async fn get_all(&self) -> Self::Result<Vec<OtherGuild>>;
    async fn get_from_guild_id(&self, guild_id: u64) -> Self::Result<Option<OtherGuild>>;
    async fn delete(&self, server_name: &str) -> Self::Result<OtherGuild>;
}

pub trait OtherTimesRepository {
    type Result<T>;

    async fn upsert(&self, other_times: OtherTimes) -> Self::Result<OtherTimes>;
    async fn get(&self, server_name: &str, member_id: u64) -> Self::Result<Option<OtherTimes>>;
    async fn get_all(&self) -> Self::Result<Vec<OtherTimes>>;
    async fn get_from_member_id(&self, member_id: u64) -> Self::Result<Vec<OtherTimes>>;
    async fn delete(&self, server_name: &str, member_id: u64) -> Self::Result<Option<OtherTimes>>;
}
