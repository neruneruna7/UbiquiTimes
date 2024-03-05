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
            self.private_key
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

// テスト
#[cfg(test)]
mod tests {
    use rsa::RsaPrivateKey;

    use crate::{
        bot_message::TimesSettingRequest,
        sign::{keys_gen, UbiquitimesKeyGenerator},
    };

    use super::*;

    #[test]
    fn test_create_verifer() {
        // verifierを作れるかどうかのテスト
        let public_key_pem = r"-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAvXmv+r7dVCuoJEHrDpeIczhH10jjFVibnL0AfX1TTJlOWPQvfwyh
gIdAVZnNEWP0endeuykII0kBftAi3kqMAEffCfmChWtfT8Qh0l1pUhQQtx4Ifg/d
yrNmQdYRP5/vu7ZVgA/s4xJEz2v50WyXZU4D0++0byI+35oYT2yrKcW7vuJmTQ4k
crZOQ1JWDVzl4AyDLrq67WiAZKufHhL88uQPxOanSaI+trjaGemSi1Vr9aupOB1E
MzrO6n6oVsTw7eDvFZfKvlf9J/8ZQsrX+/SovTVYPbqovVwtWFOQJu9fxoLN3/SS
AAXQVhGIbJdfPNYx/jiyjod6PrYHNFPrQwIDAQAB
-----END RSA PUBLIC KEY-----";
        let _verifier = UbiquitimesPublicKey::from_pem(public_key_pem).unwrap();
    }

    #[test]
    fn sign_and_verify() {
        // 署名と検証のテスト
        let key_pair = keys_gen::RsaKeyGenerator::new().generate_keys().unwrap();
        let pems = key_pair.to_pem();
        let private_key = UbiquitimesPrivateKey::from_pem(&pems.private_key_pem.as_str()).unwrap();
        let public_key = UbiquitimesPublicKey::from_pem(&pems.public_key_pem.as_str()).unwrap();

        let test_times_setting_req =
            TimesSettingRequest::new(0, "0".to_string(), 0, "0".to_string());

        let claims = Claims {
            sub: 100,
            exp: 1000000000000,
            iss: "test".to_string(),
            aud: "me".to_string(),
            times_setting_req: test_times_setting_req,
        };

        let signed_token = private_key.sign(claims.clone()).unwrap();
        let verified_claims = public_key.verify(&signed_token).unwrap();

        assert_eq!(claims, verified_claims);
    }
}
