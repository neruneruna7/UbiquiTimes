use super::*;
use crate::own_server::OwnTimesData;

use anyhow::Result;
use sled::Db;

pub struct OwnTimesDataTable<'a> {
    db: &'a sled::Db,
}

impl<'a> OwnTimesDataTable<'a> {
    pub fn new(db: &'a sled::Db) -> Self {
        Self { db }
    }
}

impl<'a> SledTable for OwnTimesDataTable<'a> {
    const TABLE_NAME: &'static str = "OwnTimesDataTable";

    type SledKey = String;

    type SledValue = OwnTimesData;

    fn get_db(&self) -> &sled::Db {
        self.db
    }
}

impl OwnTimesData {
    // keyの値はmember_id
    pub fn db_upsert(&self, db: &Db) -> Result<()> {
        let own_times_table = OwnTimesDataTable::new(db);
        own_times_table.upsert(&self.member_id.to_string(), self)
    }

    pub fn db_read(db: &Db, member_id: u64) -> Result<Option<Self>> {
        let own_times_table = OwnTimesDataTable::new(db);
        own_times_table.read(&member_id.to_string())
    }

    pub fn db_read_all(db: &Db) -> Result<Vec<Self>> {
        let own_times_table = OwnTimesDataTable::new(db);
        own_times_table.read_all()
    }

    pub fn db_delete(db: &Db, member_id: u64) -> Result<()> {
        let own_times_table = OwnTimesDataTable::new(db);
        own_times_table.delete(&member_id.to_string())
    }
}