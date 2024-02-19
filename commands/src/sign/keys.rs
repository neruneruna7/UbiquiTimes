// UbiquitimesSignerとUbiquitimesVerifierトレイトを実装する
// つまるところ，署名，検証を司る

use super::{Claims, SignResult, UbiquitimesSigner, UbiquitimesVerifier};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

// たぶんこの構造体たちの場所はモジュール分けした先だな
pub struct UbiquitimesPrivateKey {
    pub private_key: rsa::RsaPrivateKey,
}

impl UbiquitimesSigner for UbiquitimesPrivateKey {
    fn sign(&self, claims: Claims) -> SignResult<String> {
        let header = Header::new(Algorithm::RS256);
        // pemを通さずに変換したいが，既にある実装をもとにしているのでひとまず今のまま
        let key = EncodingKey::from_rsa_pem(&self.private_key.to_pkcs8_pem())?;
        let token = encode(&header, &claims, &key);

        token
    }
}

// たぶんこの構造体たちの場所はモジュール分けした先だな
pub struct UbiquitimesPublicKey {
    pub public_key: rsa::RsaPublicKey,
}

impl UbiquitimesVerifier for UbiquitimesPublicKey {
    fn verify(&self, signed_token: &str) -> SignResult<Claims> {
        let header = Header::new(Algorithm::RS256);
        let key = DecodingKey::from_rsa_pem(&self.public_key.to_pkcs8_pem()?)?;
        let validation = Validation::new(Algorithm::RS256);

        let claims = decode::<Claims>(signed_token, &key, &validation)?;

        Ok(claims.claims)
    }
}
