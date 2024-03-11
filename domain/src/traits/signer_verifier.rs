use crate::models::sign::Claims;

pub trait UbiquitimesSigner {
    type Error;
    // 引数として渡すデータはまだ確定していない
    // 前に書いた自分のコードを読まなければ...
    fn sign(&self, claims: Claims) -> Result<String, Self::Error>;
}

pub trait UbiquitimesVerifier {
    type Error;

    fn verify(&self, signed_token: &str) -> Result<Claims, Self::Error>;
}

trait TesTrait {
    type Result<T>;

    fn tes(&self) -> Self::Result<String>;
}
