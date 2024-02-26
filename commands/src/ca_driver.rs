// 他サーバの公開鍵を取得するのに必要なもの

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// クソ雑オレオレ認証局モドキへのドライバ
pub mod my_ca_driver;

#[derive(Debug, Error)]
pub enum CaDriverError {
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub type CaDriverResult<T> = Result<T, CaDriverError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyAndWebhook {
    guild_id: u64,
    public_key: String,
    manage_webhook: String,
}

pub trait CaDriver {
    async fn get_key_webhook(&self, guild_id: u64) -> CaDriverResult<KeyAndWebhook>;
}
