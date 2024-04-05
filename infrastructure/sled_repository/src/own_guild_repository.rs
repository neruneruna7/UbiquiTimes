use crate::sled_table::{SledTable, SledTableError, SledTableResult};
use domain::{models::guild_data::OwnGuild, thiserror, traits::repositorys::OwnGuildRepository};

#[derive(Debug, thiserror::Error)]
pub enum SledOwnGuildRepositoryError {
    #[error("SledError: {0}")]
    SledError(#[from] sled::Error),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("StringError: {0}")]
    StringError(#[from] std::string::FromUtf8Error),
    #[error("SledTableError: {0}")]
    SledTableError(#[from] SledTableError),
    #[error("OwnGuildNotFound: {0}")]
    OwnGuildNotFound(#[from] OwnGuildNotFound),
}

#[derive(Debug, Clone)]
pub struct OwnGuildNotFound;

impl std::fmt::Display for OwnGuildNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OwnGuildNotFound: Haven't you registered yet?")
    }
}

impl std::error::Error for OwnGuildNotFound {}

pub type SledOwnGuildResult<T> = Result<T, SledOwnGuildRepositoryError>;

#[derive(Debug)]
pub struct SledOwnGuildRepository {
    db: sled::Db,
}

impl SledOwnGuildRepository {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }
}

impl OwnGuildRepository for SledOwnGuildRepository {
    type Result<T> = SledOwnGuildResult<T>;
    // keyを引数にとる必要はない
    // 内部で読んでるseldtableトレイトの都合で引数を取る関数を使っているが，
    // それは適当に空の文字列をわたしておく
    async fn upsert(&self, own_server: OwnGuild) -> Self::Result<OwnGuild> {
        let own_server_table = OwnGuildTable::new(&self.db);
        own_server_table.upsert(&own_server.guild_name, &own_server)?;
        Ok(own_server)
    }

    async fn get(&self) -> Self::Result<OwnGuild> {
        let own_server_table = OwnGuildTable::new(&self.db);
        let data = own_server_table.read(&String::new())?;
        // NoneのときはErrを返す
        match data {
            Some(data) => Ok(data),
            None => Err(SledOwnGuildRepositoryError::OwnGuildNotFound(
                OwnGuildNotFound,
            )),
        }
    }

    async fn delete(&self) -> Self::Result<OwnGuild> {
        let own_server_table = OwnGuildTable::new(&self.db);
        let data = own_server_table.read(&String::new())?;
        own_server_table.delete(&String::new())?;

        // NoneのときはErrを返す
        match data {
            Some(data) => Ok(data),
            None => Err(SledOwnGuildRepositoryError::OwnGuildNotFound(
                OwnGuildNotFound,
            )),
        }
    }
}

struct OwnGuildTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OwnGuildTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

const KEY_NAME: &str = "OwnGuild";
impl<'a> SledTable for OwnGuildTable<'a> {
    const TABLE_NAME: &'static str = "OwnGuildDataTable";
    // 自身のサーバーデータは常に1つしかない
    // keyにどんな値を渡したとしても同じキーを使うように
    // そのためにトレイトのデフォルト実装を書き換えている

    type SledKey = String;

    type SledValue = OwnGuild;

    fn get_db(&self) -> &sled::Db {
        self.db
    }

    fn upsert(&self, _key: &Self::SledKey, value: &Self::SledValue) -> SledTableResult<()> {
        let key = KEY_NAME;
        let value = serde_json::to_string(value)?;
        let byte_key = value.as_bytes();
        let db = self.get_db();
        db.open_tree(Self::TABLE_NAME)?.insert(key, byte_key)?;
        Ok(())
    }

    fn read(&self, _key: &Self::SledKey) -> SledTableResult<Option<Self::SledValue>> {
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

    fn delete(&self, _key: &Self::SledKey) -> SledTableResult<()> {
        let db = self.get_db();
        let byte_key = KEY_NAME;
        db.open_tree(Self::TABLE_NAME)?.remove(byte_key)?;
        Ok(())
    }
}
