use crate::models::sign::Claims;

pub trait UtSigner {
    type Result<T>;
    // 引数として渡すデータはまだ確定していない
    // 前に書いた自分のコードを読まなければ...
    fn sign(&self, claims: Claims) -> Self::Result<String>;
}

pub trait UtVerifier {
    type Result<T>;

    fn verify(&self, signed_token: &str) -> Self::Result<Claims>;
}

pub trait UtKeyPairGenerator {
    type Result<T>;
    type KeyPair;

    fn generate_key_pair(&self) -> Self::Result<Self::KeyPair>;
}
