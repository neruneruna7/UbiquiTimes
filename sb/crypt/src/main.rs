// use rsa::sha2::Sha256;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

// use jsonwebtoken_rustcrypto::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::RsaPrivateKey;
// use rsa::pss::{SigningKey, VerifyingKey};
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // 送信元サーバ名
    pub iss: String,
    // GUILD_ID
    pub sub: String,
    // 送信先サーバ名
    pub aud: String,
    pub exp: usize,
    pub cmdind: String,
}

impl Claims {
    pub fn new(iss: &str, sub: &str, aud: &str, cmdind: &str) -> Claims {
        let iss = iss.to_string();
        let sub = sub.to_string();
        let aud = aud.to_string();
        let exp = 10000000000;
        let cmdind = cmdind.to_string();
        Self {
            iss,
            sub,
            aud,
            exp,
            cmdind,
        }
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let priv_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let pub_key = priv_key.to_public_key();

    let priv_key_pem = priv_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let pub_key_pem = pub_key.to_public_key_pem(LineEnding::LF).unwrap();

    let my_claims = Claims::new("nann", "22222222", "test", "cmdkind");

    let token = encode(
        &Header::new(Algorithm::RS256),
        &my_claims,
        &EncodingKey::from_rsa_pem(priv_key_pem.as_bytes()).unwrap(),
    )
    .unwrap();
    println!("Token: {}", token);

    let token = decode::<Claims>(
        &token,
        &DecodingKey::from_rsa_pem(pub_key_pem.as_bytes()).unwrap(),
        &Validation::new(Algorithm::RS256),
    )
    .unwrap();
    println!("Token: {:?}", token.claims);
}
