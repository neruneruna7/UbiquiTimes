use crate::{bot_communicate::TimesUbiquiSettingSend, global_data::Context};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    // 送信元サーバ名
    pub iss: String,
    // GUILD_ID
    pub sub: u64,
    // 送信先サーバ名
    pub aud: String,
    pub exp: usize,
    pub times_ubiqui_setting_send: TimesUbiquiSettingSend,
}

impl Claims {
    pub fn new(
        iss: &str,
        sub: u64,
        aud: &str,
        times_ubiqui_setting_send: TimesUbiquiSettingSend,
    ) -> Claims {
        let iss = iss.to_string();
        let aud = aud.to_string();
        let exp = 10000000000;
        Self {
            iss,
            sub,
            aud,
            exp,
            times_ubiqui_setting_send,
        }
    }
}

pub(crate) async fn register_public_key_ctx_data(
    guild_id: u64,
    public_key_pem: String,
    ctx: &Context<'_>,
) -> anyhow::Result<()> {
    let mut public_key_pem_hashmap = ctx.data().public_key_pem_hashmap.write().await;

    public_key_pem_hashmap.insert(guild_id, public_key_pem);
    Ok(())
}
