use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sled::Db;

fn main() -> Result<()> {
    let db = sled::open("my_database")?;
    let my_table = MyTable::new(&db);
    let key = "my_key".to_string();
    let value = MyValue::new("my_id".to_string(), "some_name".to_string(), 1);
    my_table.upsert(&key, &value)?;

    let key = "my_key2".to_string();
    my_table.upsert(&key, &value)?;
    let retrieved_value = my_table.read(&key)?.unwrap();
    assert_eq!(value, retrieved_value);

    let all_values = my_table.read_all()?;

    println!("{:?}, {:?}", value, retrieved_value);

    println!("{:?}", all_values);
    Ok(())
}
trait SledTable {
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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct MyValue {
    id: String,
    name: String,
    number: i64,
}

impl MyValue {
    fn new(id: String, name: String, number: i64) -> Self {
        Self { id, name, number }
    }
}

struct MyTable<'a> {
    db: &'a Db,
}

impl<'a> MyTable<'a> {
    fn new(db: &'a Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for MyTable<'a> {
    const TABLE_NAME: &'static str = "MyTable";
    type SledKey = String;
    type SledValue = MyValue;
    fn get_db(&self) -> &Db {
        self.db
    }
}

// [dependencies]
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1"
// serde_urlencoded = "0.7.0"
// sled = "0.34"
// anyhow = "1.0"
