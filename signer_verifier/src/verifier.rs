use domain::models::sign::Claims;
use domain::thiserror;
use domain::traits::signer_verifier::{UtSigner, UtVerifier};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::{
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey},
    pkcs8::{DecodePrivateKey, EncodePrivateKey},
};

#[derive(thiserror::Error, Debug)]
pub enum VerifyError {
    #[error("Rsa pkcs1 error: {0}")]
    RsaPkcs1Error(#[from] rsa::pkcs1::Error),
    #[error("Jws error: {0}")]
    JwsError(#[from] jsonwebtoken::errors::Error),
}

pub type VerifyResult<T> = Result<T, VerifyError>;

// たぶんこの構造体たちの場所はモジュール分けした先だな
pub struct UbiquitimesPublicKey {
    pub public_key: rsa::RsaPublicKey,
}

impl UbiquitimesPublicKey {
    pub fn new(public_key: rsa::RsaPublicKey) -> Self {
        Self { public_key }
    }
    pub fn from_pem(pem: &str) -> VerifyResult<Self> {
        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(pem)?;
        Ok(Self { public_key })
    }
}

impl UtVerifier for UbiquitimesPublicKey {
    type Result<T> = VerifyResult<T>;
    fn verify(&self, signed_token: &str) -> Self::Result<Claims> {
        let _header = Header::new(Algorithm::RS256);
        let key = DecodingKey::from_rsa_pem(
            self.public_key
                .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?
                .as_bytes(),
        )?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;

        let claims = decode::<Claims>(signed_token, &key, &validation)?;

        Ok(claims.claims)
    }
}
