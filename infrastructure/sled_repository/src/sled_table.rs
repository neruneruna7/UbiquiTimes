use domain::{
    serde::{de::DeserializeOwned, Serialize},
    thiserror,
};
use sled::Db;

#[derive(Debug, thiserror::Error)]
pub enum SledTableError {
    #[error("Sled error: {0}")]
    SledError(#[from] sled::Error),
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("String error: {0}")]
    StringError(#[from] std::string::FromUtf8Error),
    
}

pub type SledTableResult<T> = std::result::Result<T, SledTableError>;

pub trait SledTable {
    const TABLE_NAME: &'static str;
    type SledKey: AsRef<[u8]>;
    type SledValue: Serialize + DeserializeOwned;
    fn get_db(&self) -> &Db;
    fn upsert(&self, key: &Self::SledKey, value: &Self::SledValue) -> SledTableResult<()> {
        let key = key.as_ref();
        let value = serde_json::to_string(value)?;
        let byte_key = value.as_bytes();
        let db = self.get_db();
        db.open_tree(Self::TABLE_NAME)?.insert(key, byte_key)?;
        Ok(())
    }

    fn read(&self, key: &Self::SledKey) -> SledTableResult<Option<Self::SledValue>> {
        let db = self.get_db();
        let byte_key = key.as_ref();
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

    fn read_all(&self) -> SledTableResult<Vec<Self::SledValue>> {
        let db = self.get_db();
        let mut ret = Vec::new();
        let tree = db.open_tree(Self::TABLE_NAME)?;
        for item in tree.iter() {
            let (_key, value) = item?;
            let string = String::from_utf8(value.to_vec())?;
            let value = serde_json::from_str::<Self::SledValue>(&string)?;
            ret.push(value);
        }
        Ok(ret)
    }

    fn delete(&self, key: &Self::SledKey) -> SledTableResult<()> {
        let db = self.get_db();
        let byte_key = key.as_ref();
        db.open_tree(Self::TABLE_NAME)?.remove(byte_key)?;
        Ok(())
    }
}
