
use std::fmt::Display;

use domain::{models::guild_data::{OtherTimes, OwnGuild}, thiserror, traits::repositorys::{OtherTimesRepository, OwnGuildRepository}};
use shuttle_persist::PersistInstance;

#[derive(Debug, thiserror::Error)]
pub enum PersistOtherTimesRepositoryError {
    #[error("ShuttlePershistError: {0}")]
    ShuttlePersistError(#[from] shuttle_persist::PersistError),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("StringError: {0}")]
    StringError(#[from] std::string::FromUtf8Error),
}

#[derive(Debug)]
pub struct PersistOtherTimesRepository {
    db: PersistInstance,
}

impl PersistOtherTimesRepository {
    pub fn new(db: PersistInstance) -> Self {
        Self { db }
    }
}

struct Key {
    member_id: u64,
    guild_id: u64,
}

// stringへの変換を実装するためのDisplayトレイト
impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.member_id, self.guild_id)
    }
}

impl Key {
    fn new(member_id: u64, guild_id: u64) -> Self {
        Self { member_id, guild_id }
    }
}



impl OtherTimesRepository for PersistOtherTimesRepository {
    type Result<T> = Result<T, PersistOtherTimesRepositoryError>;
    
    fn upsert(&self, other_times: OtherTimes) -> Self::Result<OtherTimes> {
        let key = Key::new(other_times.src_member_id, other_times.dst_guild_id);
        self.db.save(&key.to_string(), &other_times)?;
        Ok(other_times)
    }
    
    fn get(&self, guild_id: u64, member_id: u64) -> Self::Result<Option<OtherTimes>> {
        let key = Key::new(member_id, guild_id);
        let data = self.db.load::<OtherTimes>(&key.to_string())?;
        Ok(Some(data))
    }
    
    fn get_all(&self) -> Self::Result<Vec<OtherTimes>> {
        todo!()
    }
    
    fn get_from_member_id(&self, member_id: u64) -> Self::Result<Vec<OtherTimes>> {
        todo!()
    }
    
    fn delete(&self, guild_id: u64, member_id: u64) -> Self::Result<Option<OtherTimes>> {
        todo!()
    }
    
}
