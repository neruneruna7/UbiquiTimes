// use rsa::sha2::Sha256;
use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

// use jsonwebtoken_rustcrypto::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs8::{EncodePrivateKey, DecodePrivateKey, EncodePublicKey, DecodePublicKey, LineEnding};
// use rsa::pss::{SigningKey, VerifyingKey};
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

fn main(){    
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let priv_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let pub_key = priv_key.to_public_key();

    let priv_key_pem = priv_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let pub_key_pem = pub_key.to_public_key_pem(LineEnding::LF).unwrap();
    
    let my_claims = Claims {
        sub: "me".to_owned(),
        company: "ACME".to_owned(),
        exp: 10000000000,
    };

    let token = encode(&Header::new(Algorithm::RS256), &my_claims, &EncodingKey::from_rsa_pem(priv_key_pem.as_bytes()).unwrap()).unwrap();
    println!("Token: {}", token);
    
    let token = decode::<Claims>(&token, &DecodingKey::from_rsa_pem(pub_key_pem.as_bytes()).unwrap(), &Validation::new(Algorithm::RS256)).unwrap();
    println!("Token: {:?}", token.claims);
}