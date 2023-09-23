use rsa::RsaPrivateKey;
use rsa::pkcs1v15::{SigningKey, VerifyingKey};
use rsa::signature::{Keypair, RandomizedSigner, SignatureEncoding, Verifier};
use rsa::sha2::{Digest, Sha256};

use spki::EncodePublicKey;
use spki::DecodePublicKey;
use base64ct::LineEnding;

fn main() {
    println!("Hello, world!");
    let mut rng = rand::thread_rng();

    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let signing_key = SigningKey::<Sha256>::new(private_key);
    let verifying_key = signing_key.verifying_key();
    

    //sign
    let data = b"Hello, world!";
    let signature = signing_key.sign_with_rng(&mut rng, data);
    println!("signature: {}", signature.to_string()); 

    //verify
    let a = verifying_key.verify(data, &signature).expect("failed to verify");
    println!("verify: {:?}", a);

    // let a:i32 = signature;
}
