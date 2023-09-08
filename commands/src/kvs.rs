use sled::Db;

// kvsへの保存と取得を簡単に行うtraitを定義する
// sled::Dbを拡張する
// 命名は悩みどころ

pub(crate)trait EzKvs<T: serde::Serialize + for<'a> serde::Deserialize<'a>> {
    fn ez_set(&self, key: &str, value: &T) -> anyhow::Result<()>;
    fn ez_get(&self, key: &str) -> anyhow::Result<Option<T>>;
}

impl<T: serde::Serialize + for<'a> serde::Deserialize<'a>>  EzKvs<T> for Db {
    fn ez_set(&self, key: &str, value: &T) -> anyhow::Result<()> {
        let json = serde_json::to_string(&value).unwrap();
        // let key = key.as_bytes();
        let value = json.as_bytes();

        self.insert(key, value)?;

        Ok(())
    }

    fn ez_get(&self, key: &str) -> anyhow::Result<Option<T>> {
        // let key = key.as_bytes();
        let result = self.get(key)?;
        if let Some(ivec) = result {
            let string_value = String::from_utf8(ivec.to_vec())?;
            let json = serde_json::from_str(string_value.as_str())?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }
}
