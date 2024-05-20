use std::{collections::HashMap, hash::Hash};

use domain::thiserror;
use serde::{de::DeserializeOwned, Serialize};
use shuttle_persist::PersistInstance;

#[derive(Debug, thiserror::Error)]
pub enum PersistTableError {
    #[error("ShuttlePersist error: {0}")]
    ShuttlePersistError(#[from] shuttle_persist::PersistError),
    // #[error("Serde error: {0}")]
    // SerdeError(#[from] serde_json::Error),
    // #[error("String error: {0}")]
    // StringError(#[from] std::string::FromUtf8Error),
}

pub type PersistTableResult<T> = std::result::Result<T, PersistTableError>;


pub trait PersistTable {
    const TABLE_NAME: &'static str;
    type PersistKey: Serialize + DeserializeOwned + Eq + Hash;
    type PersistValue: Serialize + DeserializeOwned + Clone;
    fn get_db(&self) -> &PersistInstance;
    fn open_table(&self) -> PersistTableResult<HashMap<Self::PersistKey, Self::PersistValue>> {
        let db = self.get_db();
        let table = match db.load::<HashMap<Self::PersistKey, Self::PersistValue>>(Self::TABLE_NAME) {
            Ok(t) => t,
            Err(_) =>  HashMap::new(),
        };
        Ok(table)
    }
    // 各種実装が安全なのかは確かめられていない
    // マルチスレッド環境だとうまくいかない

    fn upsert(&self, key: Self::PersistKey, value: Self::PersistValue) -> PersistTableResult<()> {
        let table = self.open_table()?;
        let mut table = table;
        table.insert(key, value);
        self.get_db().save(Self::TABLE_NAME, table)?;

        Ok(())
    }

    fn read(&self, key: Self::PersistKey) -> PersistTableResult<Option<Self::PersistValue>> {
        let table = self.open_table()?;
        let value = table.get(&key).cloned();
        Ok(value)
    }

    fn read_all(&self) -> PersistTableResult<Vec<Self::PersistValue>> {
        let table = self.open_table()?;
        let values = table.values().cloned().collect();
        Ok(values)
    }

    fn delete(&self, key: Self::PersistKey) -> PersistTableResult<()> {
        let table = self.open_table()?;
        let mut table = table;
        table.remove(&key);
        self.get_db().save(Self::TABLE_NAME, table)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestPersist {
        db: PersistInstance,
    }
    
    impl PersistTable for TestPersist {
        const TABLE_NAME: &'static str = "test_table";
    
        type PersistKey = String;
    
        type PersistValue = String;
    
        fn get_db(&self) -> &PersistInstance {
            &self.db
        }
    }

    #[test]
    fn test_open_table() {
        // 初期化のためにディレクトリを削除
        std::fs::remove_dir_all("test_persist").unwrap();

        let db = PersistInstance::new("test_persist".into()).unwrap();
        let test_persist = TestPersist { db };

        let result_table = test_persist.open_table().unwrap();
        let test_table = HashMap::new();

        assert_eq!(result_table, test_table);
    }

    #[test]
    fn test_upsert() {
                // 初期化のためにディレクトリを削除
                std::fs::remove_dir_all("test_persist").unwrap();

        let db = PersistInstance::new("test_persist".into()).unwrap();
        let test_persist = TestPersist { db };

        // キーと値をクリア
        test_persist.open_table().unwrap().clear();

        let key = "key".to_string();
        let value = "value".to_string();

        test_persist.upsert(key.clone(), value.clone()).unwrap();

        let result_table = test_persist.open_table().unwrap();
        let mut test_table = HashMap::new();
        test_table.insert(key, value);

        assert_eq!(result_table, test_table);
    }

    #[test]
    fn test_read() {
                // 初期化のためにディレクトリを削除
                std::fs::remove_dir_all("test_persist").unwrap();
        let db = PersistInstance::new("test_persist".into()).unwrap();
        let test_persist = TestPersist { db };

        // キーと値をクリア
        test_persist.open_table().unwrap().clear();

        let key = "key".to_string();
        let value = "value".to_string();

        test_persist.upsert(key.clone(), value.clone()).unwrap();

        let result_value = test_persist.read(key.clone()).unwrap();
        let test_value = Some(value);

        assert_eq!(result_value, test_value);
    }

    #[test]
    fn test_read_all() {
                // 初期化のためにディレクトリを削除
                std::fs::remove_dir_all("test_persist").unwrap();
        let db = PersistInstance::new("test_persist".into()).unwrap();
        let test_persist = TestPersist { db };

        // キーと値をクリア
        test_persist.open_table().unwrap().clear();

        let key1 = "key1".to_string();
        let value1 = "value1".to_string();
        let key2 = "key2".to_string();
        let value2 = "value2".to_string();

        test_persist.upsert(key1.clone(), value1.clone()).unwrap();
        test_persist.upsert(key2.clone(), value2.clone()).unwrap();

        let mut result_values = test_persist.read_all().unwrap();
        let mut test_values = vec![value1, value2];

        assert_eq!(result_values.sort(), test_values.sort());
    }

    #[test]
    fn test_delete() {
                // 初期化のためにディレクトリを削除
                std::fs::remove_dir_all("test_persist").unwrap();
        let db = PersistInstance::new("test_persist".into()).unwrap();
        let test_persist = TestPersist { db };

        // キーと値をクリア
        test_persist.open_table().unwrap().clear();

        let key = "key".to_string();
        let value = "value".to_string();

        test_persist.upsert(key.clone(), value.clone()).unwrap();

        test_persist.delete(key.clone()).unwrap();

        let result_table = test_persist.open_table().unwrap();
        let test_table = HashMap::new();

        assert_eq!(result_table, test_table);
    }
}