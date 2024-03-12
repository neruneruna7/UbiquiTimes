use domain::models::sign::KeyAndWebhook;
// クソ雑オレオレ認証局モドキへのドライバ
use domain::thiserror;
use domain::traits::ca_driver::CaDriver;
use reqwest;

#[derive(Debug, thiserror::Error)]
pub enum MyCaDriverError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

pub struct MyCaDriver;

impl Default for MyCaDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl MyCaDriver {
    pub fn new() -> Self {
        MyCaDriver
    }
}

// URLを定義しておく
///
/// GET https://steel-borne.shuttleapp.rs/ubiquitimes/v1/oreoreca/get/{guild_id}
/// あえてここに書くことにした
const BASE_URL: &str = "https://steel-borne.shuttleapp.rs/ubiquitimes/v1/oreoreca/get/";

impl CaDriver for MyCaDriver {
    type Error = MyCaDriverError;
    async fn get_key_webhook(&self, guild_id: u64) -> Result<KeyAndWebhook, Self::Error> {
        // URLを組み立て，GETリクエストを送る
        let url = format!("{}{}", BASE_URL, guild_id);
        let response = reqwest::get(&url).await?.text().await?;

        // レスポンスをデシリアライズして返す
        let key_and_webhook: KeyAndWebhook = serde_json::from_str(&response)?;

        Ok(key_and_webhook)
    }
}
