use rsa::pkcs8::der::zeroize::Zeroizing;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::{RsaPrivateKey, RsaPublicKey};

/// RSA-2048の鍵ペアを生成する
pub fn generate_keypair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let public_key = private_key.to_public_key();

    (private_key, public_key)
}

/// キーペアをPEM形式を入れるための構造体
pub struct KeyPairPem {
    pub private_key_pem: Zeroizing<String>,
    pub public_key_pem: String,
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
