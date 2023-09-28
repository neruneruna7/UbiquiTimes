use rsa::pkcs8::der::zeroize::Zeroizing;
use rsa::pkcs8::{
    DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding,
};
use rsa::{RsaPrivateKey, RsaPublicKey};

pub fn generate_keypair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let public_key = private_key.to_public_key();

    (private_key, public_key)
}

pub struct KeyPair_pem {
    private_key_pem: Zeroizing<String>,
    public_key_pem: String,
}

pub fn keypair_to_pem(private_key: &RsaPrivateKey, public_key: &RsaPublicKey) -> KeyPair_pem {
    let private_key_pem = private_key.to_pkcs8_pem(LineEnding::LF).unwrap();

    let public_key_pem = public_key.to_public_key_pem(LineEnding::LF).unwrap();

    KeyPair_pem {
        private_key_pem,
        public_key_pem,
    }
}
