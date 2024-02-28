use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct OwnServer {
    pub guild_id: u64,
    pub server_name: String,
    pub manage_channel_id: u64,
    pub manage_webhook_url: String,
    pub private_key_pem: String,
    pub public_key_pem: String,
}

impl OwnServer {
    pub fn new(
        guild_id: u64,
        server_name: &str,
        manage_channel_id: u64,
        manage_webhook_url: &str,
        private_key_pem: &str,
        public_key_pem: &str,
    ) -> Self {
        Self {
            guild_id,
            server_name: server_name.to_string(),
            manage_channel_id,
            manage_webhook_url: manage_webhook_url.to_string(),
            private_key_pem: private_key_pem.to_string(),
            public_key_pem: public_key_pem.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct OwnTimes {
    pub member_id: u64,
    pub member_name: String,
    pub channel_id: u64,
    pub times_webhook_url: String,
}

impl OwnTimes {
    pub fn new(
        member_id: u64,
        member_name: &str,
        channel_id: u64,
        times_webhook_url: &str,
    ) -> Self {
        Self {
            member_id,
            member_name: member_name.to_string(),
            channel_id,
            times_webhook_url: times_webhook_url.to_string(),
        }
    }
}
