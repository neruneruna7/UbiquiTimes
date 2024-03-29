// むむ，うまいこと構成するの難しいな．
// あとでリファクタリングするかもしれない．
// pub mod claims;
pub mod keys;
pub mod keys_gen;

use crate::bot_message::TimesSettingRequest;
use crate::other_server::OtherServer;
use crate::own_server::OwnServer;
use thiserror::Error;

pub use keys::UbiquitimesPrivateKey;
pub use keys::UbiquitimesPublicKey;

#[derive(Debug, Error)]
pub enum SignError {
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    // 他のエラータイプもここに追加できます
    #[error("JsonWebToken error: {0}")]
    JsonWebTokenError(#[from] jsonwebtoken::errors::Error),
    #[error("Rsa pkcs8 error: {0}")]
    RsaError(#[from] rsa::pkcs8::Error),
    #[error("Rsa pkcs1 error: {0}")]
    RsaPkcs1Error(#[from] rsa::pkcs1::Error),
}

pub type SignResult<T> = Result<T, SignError>;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
// なんだったか忘れたけど，何かに基づいてClaim型を定義した
// 署名のときの云々があったはず...
pub struct Claims {
    // 送信元サーバ名
    pub iss: String,
    // GUILD_ID
    // ...どっちのだっけ？
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

    /// リクエスト送信時につかうClaimsをサーバーデータから生成する
    pub fn from_servers_for_req(
        own_server: &OwnServer,
        other_server: &OtherServer,
        times_setting_req: TimesSettingRequest,
    ) -> Self {
        let iss = own_server.server_name.clone();
        let sub = own_server.guild_id;
        let aud = other_server.server_name.clone();
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

pub struct UbiquitimesKeys {
    pub private_key: UbiquitimesPrivateKey,
    pub public_key: UbiquitimesPublicKey,
}

pub trait UbiquitimesKeyGenerator {
    fn generate_keys(&self) -> SignResult<UbiquitimesKeys>;
}

pub trait UbiquitimesSigner {
    // 引数として渡すデータはまだ確定していない
    // 前に書いた自分のコードを読まなければ...
    fn sign(&self, claims: Claims) -> SignResult<String>;
}

pub trait UbiquitimesVerifier {
    fn verify(&self, signed_token: &str) -> SignResult<Claims>;
}
