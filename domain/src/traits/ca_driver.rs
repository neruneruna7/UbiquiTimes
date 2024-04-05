use crate::models::sign::KeyAndWebhook;

pub trait CaDriver {
    type Error;
    fn get_key_webhook(
        &self,
        guild_id: u64,
    ) -> impl std::future::Future<Output = Result<KeyAndWebhook, Self::Error>> + Send;
}
