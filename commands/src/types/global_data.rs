use anyhow::Error;

use poise::serenity_prelude::RwLock;
use sqlx::SqlitePool;
use std::sync::Arc;

// Types used by all command functions
// すべてのコマンド関数で使用される型
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
pub struct Data {
    pub connection: Arc<SqlitePool>,
    pub master_webhook_url: RwLock<String>,
    // 秘密鍵はここにはのせない
    pub public_key_pem: RwLock<String>,
}
