use domain::{models::guild_data::OwnGuild, thiserror, traits::repositorys::OwnGuildRepository};
use shuttle_persist::PersistInstance;

#[derive(Debug, thiserror::Error)]
pub enum PersistOwnGuildRepositoryError {
    #[error("ShuttlePershistError: {0}")]
    ShuttlePersistError(#[from] shuttle_persist::PersistError),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("StringError: {0}")]
    StringError(#[from] std::string::FromUtf8Error),
    // #[error("OwnGuildNotFound: {0}")]
    // OwnGuildNotFound(#[from] OwnGuildNotFound),
}

#[derive(Debug)]
pub struct PersistOwnGuildRepository {
    db: PersistInstance,
}

impl PersistOwnGuildRepository {
    pub fn new(db: PersistInstance) -> Self {
        Self { db }
    }
}

// キーは常に一定で問題ない
// 必ず1つしかないので
const KEY: &str = "own_guild";

impl OwnGuildRepository for PersistOwnGuildRepository {
    type Result<T> = Result<T, PersistOwnGuildRepositoryError>;

    fn upsert(&self, own_server: OwnGuild) -> Self::Result<OwnGuild> {
        self.db.save(KEY, &own_server)?;
        Ok(own_server)
    }

    fn get(&self) -> Self::Result<OwnGuild> {
        let data = self.db.load::<OwnGuild>(KEY)?;
        Ok(data)
    }

    fn delete(&self) -> Self::Result<OwnGuild> {
        let data = self.get()?;
        self.db.remove(KEY)?;
        Ok(data)
    } // 適切なエラータイプに置き換えてください
}
