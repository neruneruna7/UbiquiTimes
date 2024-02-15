use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use sled::Db;

pub trait SledTable {
    const TABLE_NAME: &'static str;
    type SledKey: AsRef<[u8]>;
    type SledValue: Serialize + DeserializeOwned;
    fn get_db(&self) -> &Db;
    fn upsert(&self, key: &Self::SledKey, value: &Self::SledValue) -> Result<()> {
        let key = key.as_ref();
        let value = serde_json::to_string(value)?;
        let byte_key = value.as_bytes();
        let db = self.get_db();
        db.open_tree(Self::TABLE_NAME)?.insert(key, byte_key)?;
        Ok(())
    }

    fn read(&self, key: &Self::SledKey) -> Result<Option<Self::SledValue>> {
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

    fn read_all(&self) -> Result<Vec<Self::SledValue>> {
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

    fn delete(&self, key: &Self::SledKey) -> Result<()> {
        let db = self.get_db();
        let byte_key = key.as_ref();
        db.open_tree(Self::TABLE_NAME)?.remove(byte_key)?;
        Ok(())
    }
}
