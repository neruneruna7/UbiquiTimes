// クソ雑オレオレ認証局モドキへのドライバ
use super::{CaDriver, CaDriverResult, KeyAndWebhook};
use reqwest;

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
const BASE_URL: &str = "https://steel-borne.shuttleapp.rs/ubiquitimes/v1/oreoreca/get/";

impl CaDriver for MyCaDriver {
    async fn get_key_webhook(&self, guild_id: u64) -> CaDriverResult<super::KeyAndWebhook> {
        // URLを組み立て，GETリクエストを送る
        let url = format!("{}{}", BASE_URL, guild_id);
        let response = reqwest::get(&url).await?.text().await?;

        // レスポンスをデシリアライズして返す
        let key_and_webhook: KeyAndWebhook = serde_json::from_str(&response)?;

        Ok(key_and_webhook)
    }
}
