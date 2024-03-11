use crate::models::sign::Claims;

pub trait UtSigner {
    type Error;
    // 引数として渡すデータはまだ確定していない
    // 前に書いた自分のコードを読まなければ...
    fn sign(&self, claims: Claims) -> Result<String, Self::Error>;
}

pub trait UtVerifier {
    type Error;

    fn verify(&self, signed_token: &str) -> Result<Claims, Self::Error>;
}
