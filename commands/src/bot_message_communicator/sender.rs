pub struct WebhookSender {
    pub webhook_url: String,
}

impl WebhookSender {
    pub fn new(webhook_url: &str) -> Self {
        let webhook_url = webhook_url.to_string();
        Self { webhook_url }
    }
}
