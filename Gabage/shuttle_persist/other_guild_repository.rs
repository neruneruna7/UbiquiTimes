use std::collections::HashMap;

use domain::models::guild_data::{self, OtherGuild};
use domain::traits::repositorys::OtherGuildRepository;
use domain::{thiserror, tracing};
use shuttle_persist::PersistInstance;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum PersistOtherGuildRepositoryError {
    #[error("ShuttlePersistError: {0}")]
    ShuttlePershistError(#[from] shuttle_persist::PersistError),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("StringError: {0}")]
    StringError(#[from] std::string::FromUtf8Error),
}

#[derive(Debug)]
pub struct PersistOtherGuildRepository {
    db: PersistInstance,
}

impl PersistOtherGuildRepository {
    /// Creates a new [`PersistOtherGuildRepository`].
    pub fn new(db: PersistInstance) -> Self {
        Self { db }
    }
}

// get_allを実装するために，テーブル的なもので分ける
const TABLE_KEY: &str = "other_guild";
type OtherGuildTable = HashMap<String, OtherGuild>;

impl OtherGuildRepository for PersistOtherGuildRepository {
    type Result<T> = Result<T, PersistOtherGuildRepositoryError>;

    fn upsert(&self, other_server: OtherGuild) -> Self::Result<OtherGuild> {
        let table_result = self.db.load::<OtherGuildTable>(TABLE_KEY);
        let mut table = match table_result {
            Ok(t) => t,
            // テーブルがまだ作成されていなければ，新しく作成する
            Err(e) => match e {
                shuttle_persist::PersistError::InvalidKey => OtherGuildTable::new(),
                _ => return Err(PersistOtherGuildRepositoryError::ShuttlePershistError(e)),
            },
        };
        let _old_guild = table.insert(other_server.guild_name.clone(), other_server.clone());

        self.db.save(TABLE_KEY, table)?;
        Ok(other_server)
    }

    fn get(&self, server_name: &str) -> Self::Result<Option<OtherGuild>> {
        let table = match self.db.load::<OtherGuildTable>(TABLE_KEY) {
            Ok(t) => t,
            Err(e) => match e {
                shuttle_persist::PersistError::InvalidKey => return Ok(None),
                _ => return Err(PersistOtherGuildRepositoryError::ShuttlePershistError(e))
            },
        };
        let data = table.get(server_name).cloned();
        Ok(data)
    }

    fn get_all(&self) -> Self::Result<Vec<OtherGuild>> {
        todo!()
    }

    fn get_from_guild_id(&self, guild_id: u64) -> Self::Result<Option<OtherGuild>> {
        todo!()
    }

    fn delete(&self, server_name: &str) -> Self::Result<OtherGuild> {
        todo!()
    }
}
