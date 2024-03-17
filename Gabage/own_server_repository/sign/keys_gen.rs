use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::pkcs8::der::zeroize::Zeroizing;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::{RsaPrivateKey, RsaPublicKey};

use super::keys::{UbiquitimesPrivateKey, UbiquitimesPublicKey};
use super::{SignResult, UbiquitimesKeyGenerator, UbiquitimesKeys};

/// キーペアをPEM形式を入れるための構造体
pub struct KeyPairPem {
    pub private_key_pem: Zeroizing<String>,
    pub public_key_pem: String,
}

/// キーペアを生成するときにつかう
/// 今のところ，それ以外の使い道は想定していない
///
#[derive(Clone, Copy, Debug)]
pub struct RsaKeyGenerator;

impl Default for RsaKeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RsaKeyGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl UbiquitimesKeyGenerator for RsaKeyGenerator {
    /// RSA-2048の鍵ペアを生成する
    fn generate_keys(&self) -> SignResult<UbiquitimesKeys> {
        let (private_key, public_key) = generate_keypair();
        let private_key = UbiquitimesPrivateKey { private_key };
        let public_key = UbiquitimesPublicKey { public_key };

        Ok(UbiquitimesKeys {
            private_key,
            public_key,
        })
    }
}

impl UbiquitimesKeys {
    /// キーペアをPEM形式に変換する
    pub fn to_pem(&self) -> KeyPairPem {
        keypair_to_pem(&self.private_key.private_key, &self.public_key.public_key)
    }
}

/// RSA-2048の鍵ペアを生成する
fn generate_keypair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let public_key = private_key.to_public_key();

    (private_key, public_key)
}

/// キーペアをPEM形式に変換する
pub fn keypair_to_pem(private_key: &RsaPrivateKey, public_key: &RsaPublicKey) -> KeyPairPem {
    let private_key_pem = private_key.to_pkcs8_pem(LineEnding::LF).unwrap();

    let public_key_pem = public_key.to_pkcs1_pem(LineEnding::LF).unwrap();

    KeyPairPem {
        private_key_pem,
        public_key_pem,
    }
}
