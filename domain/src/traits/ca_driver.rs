use crate::models::sign::KeyAndWebhook;

pub trait CaDriver {
    type Error;
    async fn get_key_webhook(&self, guild_id: u64) -> Result<KeyAndWebhook, Self::Error>;
}
