// UbiquitimesSignerとUbiquitimesVerifierトレイトを実装する
// つまるところ，署名，検証を司る

use super::{Claims, SignResult, UbiquitimesSigner, UbiquitimesVerifier};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::{
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey},
    pkcs8::{DecodePrivateKey, EncodePrivateKey},
};

// たぶんこの構造体たちの場所はモジュール分けした先だな
pub struct UbiquitimesPrivateKey {
    pub private_key: rsa::RsaPrivateKey,
}

impl UbiquitimesPrivateKey {
    pub fn new(private_key: rsa::RsaPrivateKey) -> Self {
        Self { private_key }
    }

    pub fn from_pem(pem: &str) -> SignResult<Self> {
        let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(pem)?;
        Ok(Self { private_key })
    }
}

impl UbiquitimesSigner for UbiquitimesPrivateKey {
    fn sign(&self, claims: Claims) -> SignResult<String> {
        let header = Header::new(Algorithm::RS256);
        // pemを通さずに変換したいが，既にある実装をもとにしているのでひとまず今のまま
        // let key = EncodingKey::from(&self.private_key);
        let key = EncodingKey::from_rsa_pem(
            &self
                .private_key
                .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?
                .as_bytes(),
        )?;
        let token = encode(&header, &claims, &key)?;

        Ok(token)
    }
}

// たぶんこの構造体たちの場所はモジュール分けした先だな
pub struct UbiquitimesPublicKey {
    pub public_key: rsa::RsaPublicKey,
}

impl UbiquitimesPublicKey {
    pub fn new(public_key: rsa::RsaPublicKey) -> Self {
        Self { public_key }
    }
    pub fn from_pem(pem: &str) -> SignResult<Self> {
        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(pem)?;
        Ok(Self { public_key })
    }
}

impl UbiquitimesVerifier for UbiquitimesPublicKey {
    fn verify(&self, signed_token: &str) -> SignResult<Claims> {
        let _header = Header::new(Algorithm::RS256);
        let key = DecodingKey::from_rsa_pem(
            &self
                .public_key
                .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)?
                .as_bytes(),
        )?;
        let validation = Validation::new(Algorithm::RS256);

        let claims = decode::<Claims>(signed_token, &key, &validation)?;

        Ok(claims.claims)
    }
}
