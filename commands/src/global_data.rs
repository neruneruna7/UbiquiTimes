use anyhow::Error;

use poise::serenity_prelude::RwLock;
use sqlx::SqlitePool;
use std::hash::Hash;
use std::sync::Arc;

use std::collections::{HashMap, HashSet};

// Types used by all command functions
// すべてのコマンド関数で使用される型
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
pub struct Data {
    pub connection: Arc<SqlitePool>,
    pub master_webhook_url: RwLock<String>,
    // 秘密鍵はここにはのせない
    pub public_key_pem_hashmap: RwLock<HashMap<u64, String>>,
    // ユーザidと送信先ギルドid
    pub botcom_sended: RwLock<HashMap<u64, HashSet<u64>>>,
}
