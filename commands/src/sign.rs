// むむ，うまいこと構成するの難しいな．
// あとでリファクタリングするかもしれない．
pub mod claims;
pub mod keys;
pub mod keys_gen;

use crate::bot_message::{
    RequestMessage, ResponseMessage, TimesSettingRequest, TimesSettingResponce,
};
use crate::other_server::OtherServer;
use crate::own_server::OwnServer;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignError {
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    // 他のエラータイプもここに追加できます
}

pub type SignResult<T> = Result<T, SignError>;

use serde::{Deserialize, Serialize};

use self::keys_gen::UbiquitimesKeys;

#[derive(Debug, Serialize, Deserialize, Clone)]
// なんだったか忘れたけど，何かに基づいてClaim型を定義した
// 署名のときの云々があったはず...
pub struct Claims {
    // 送信元サーバ名
    pub iss: String,
    // GUILD_ID
    pub sub: u64,
    // 送信先サーバ名
    pub aud: String,
    pub exp: usize,
    pub times_setting_req: TimesSettingRequest,
}

impl Claims {
    pub fn new(iss: &str, sub: u64, aud: &str, times_setting_req: TimesSettingRequest) -> Claims {
        let iss = iss.to_string();
        let aud = aud.to_string();
        let exp = 10000000000;
        Self {
            iss,
            sub,
            aud,
            exp,
            times_setting_req,
        }
    }
}

pub trait UbiquitimesKeyGenerator {
    fn generate_keys() -> SignResult<UbiquitimesKeys>;
}

pub trait UbiquitimesSigner {
    // 引数として渡すデータはまだ確定していない
    // 前に書いた自分のコードを読まなければ...
    fn sign(&self, claims: Claims) -> SignResult<String>;
}

pub trait UbiquitimesVerifier {
    fn verify(&self, signed_token: &str) -> SignResult<Claims>;
}
