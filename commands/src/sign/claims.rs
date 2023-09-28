use crate::{
    db_query,
    types::{
        botcom::CmdKind,
        global_data::{Context, Data},
        own_server_data::ServerData,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // 送信元サーバ名
    pub iss: String,
    // GUILD_ID
    pub sub: String,
    // 送信先サーバ名
    pub aud: String,
    pub exp: usize,
    pub cmdkind: CmdKind,
}

impl Claims {
    pub fn new(iss: &str, sub: u64, aud: &str, cmdkind: CmdKind) -> Claims {
        let iss = iss.to_string();
        let sub = sub.to_string();
        let aud = aud.to_string();
        let exp = 10000000000;
        Self {
            iss,
            sub,
            aud,
            exp,
            cmdkind,
        }
    }
}

pub(crate) async fn register_public_key_ctx_data(
    server_data: ServerData,
    ctx: &Context<'_>,
) -> anyhow::Result<()> {
    let mut public_key_pem = ctx.data().public_key_pem.write().await;
    *public_key_pem = server_data.public_key_pem;
    Ok(())
}
