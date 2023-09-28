use crate::types::botcom::CmdKind;
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
    pub cmdind: CmdKind,
}

impl Claims {
    pub fn new(iss: &str, sub: &str, aud: &str, cmdind: CmdKind) -> Claims {
        let iss = iss.to_string();
        let sub = sub.to_string();
        let aud = aud.to_string();
        let exp = 10000000000;
        Self {
            iss,
            sub,
            aud,
            exp,
            cmdind,
        }
    }
}
