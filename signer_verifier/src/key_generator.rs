use domain::thiserror;
use domain::traits::signer_verifier::UtKeyPairGenerator;
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::pkcs8::der::zeroize::Zeroizing;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::{RsaPrivateKey, RsaPublicKey};

#[derive(thiserror::Error, Debug)]
pub enum KeyGeneratorError {}

pub type KeyGeneratorResult<T> = Result<T, KeyGeneratorError>;

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

pub struct RsaKeyPair {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}
impl RsaKeyPair {
    /// キーペアをPEM形式に変換する
    pub fn to_pem(&self) -> KeyPairPem {
        keypair_to_pem(&self.private_key, &self.public_key)
    }
}

impl UtKeyPairGenerator for RsaKeyGenerator {
    type Result<T> = KeyGeneratorResult<T>;
    type KeyPair = RsaKeyPair;
    /// RSA-2048の鍵ペアを生成する
    fn generate_key_pair(&self) -> Self::Result<Self::KeyPair> {
        let (private_key, public_key) = generate_keypair();
        Ok(RsaKeyPair {
            private_key,
            public_key,
        })
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
