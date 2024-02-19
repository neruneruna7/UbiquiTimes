use rsa::pkcs8::der::zeroize::Zeroizing;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::{RsaPrivateKey, RsaPublicKey};

use super::keys::{UbiquitimesPrivateKey, UbiquitimesPublicKey};
use super::{SignResult, UbiquitimesKeyGenerator};

/// キーペアをPEM形式を入れるための構造体
pub struct KeyPairPem {
    private_key_pem: Zeroizing<String>,
    public_key_pem: String,
}

/// キーペアを生成するときにつかう
/// 今のところ，それ以外の使い道は想定していない
/// 生成した後デストラクトして使ってね
/// ```
/// let UbiquitimesKeys { private_key, public_key } = UbiquitimesKeys::generate_keys().unwrap();
/// ```
pub struct UbiquitimesKeys {
    private_key: UbiquitimesPrivateKey,
    public_key: UbiquitimesPublicKey,
}
impl UbiquitimesKeyGenerator for UbiquitimesKeys {
    /// RSA-2048の鍵ペアを生成する
    fn generate_keys() -> SignResult<UbiquitimesKeys> {
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

    let public_key_pem = public_key.to_public_key_pem(LineEnding::LF).unwrap();

    KeyPairPem {
        private_key_pem,
        public_key_pem,
    }
}
