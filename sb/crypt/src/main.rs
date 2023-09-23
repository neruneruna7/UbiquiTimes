use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1v15::{SigningKey, VerifyingKey};
use rsa::signature::{Keypair, RandomizedSigner, SignatureEncoding, Verifier};
use rsa::sha2::{Digest, Sha256};
// use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, DecodePublicKey, EncodePublicKey};


use spki::EncodePublicKey;
use spki::DecodePublicKey;
use base64ct::LineEnding;

fn main() {
    println!("Hello, world!");
    let mut rng = rand::thread_rng();

    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let public_key = private_key.to_public_key();
    public_key.to_public_key_pem(LineEnding::LF).unwrap();

    // RsaPublicKey::from_public_key_pem(s)

    // let a = private_key.to_pkcs8_pem(LineEnding::LF).unwrap().as_str();

    let signing_key = SigningKey::<Sha256>::new(private_key);
    println!("signing_key: {:?}", signing_key);

    let verifying_key = signing_key.verifying_key();
    println!("verifyng_key:{:?}", verifying_key);

    // //sign
    // let data = b"Hello, world!";
    // let signature = signing_key.sign_with_rng(&mut rng, data);
    // println!("signature: {}", signature.to_string()); 

    // //verify
    // let a = verifying_key.verify(data, &signature).expect("failed to verify");
    // println!("verify: {:?}", a);


    // let binding = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    // let a = binding.as_str();
    // println!("a: {:?}", &a);

    // let binding = signing_key.to_pkcs8_der();

    // let a:i32 = signing_key;
    // let a: i32 = verifying_key;
}
