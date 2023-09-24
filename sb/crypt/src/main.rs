// use ring::{hmac, rand};
// use ring::rand::SecureRandom;
// use ring::error::Unspecified;

// use crypto::common::Key;

// use signature::{Signer, Verifier};
use rsa::RsaPrivateKey;
use rsa::pss::{BlindedSigningKey, SigningKey, VerifyingKey, Signature};
use rsa::signature::{Keypair,RandomizedSigner, SignatureEncoding, Verifier};
use rsa::sha2::{Digest, Sha256};

// use rsa::signature::rand_core::CryptoRngCore;

fn main() {
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let priv_key = rsa::RsaPrivateKey::new(&mut rng, bits).unwrap();
    let pub_key = rsa::RsaPublicKey::from(&priv_key);
    let sign_key: SigningKey<Sha256> = SigningKey::new(priv_key);
    let veri_key: VerifyingKey<Sha256> = VerifyingKey::new(pub_key);
    // let pub_key = rsa::RsaPublicKey::from(&priv_key);

    let msg = "動け...サジェスト...! まだだ...! これからもっと 面白く...";

    // なんかサジェストが機能しない なんで？？
    let signature: Signature = sign_key.sign_with_rng(&mut rng, "msg".as_bytes());

    println!("s: {:?}", &signature);

    veri_key.verify(msg.as_bytes(), &signature).expect("faild to verify");

    // let sign_key:BlindedSigningKey<Sha256> = BlindedSigningKey::new(priv_key);

    // let verifying_key = sign_key.verifying_key();


    
    
    // Verifier::verify(veri_key, "s", "");
    // let message = b"hello world";

    // let a = priv_key.sign(Pss::new::<Sha256>(), message).unwrap();
    // println!("{:?}",&a);

    // let b = pub_key.verify(Pss::new::<Sha256>(), message, &a).unwrap();








    

    // let veri_key = sign_key.verifying_key();



    // priv_key.sign(padding, digest_in)


    // let mut key_value = [0u8; 48];
    // let rng = rand::SystemRandom::new();
    // rng.fill(&mut key_value)?;
    // let key = hmac::Key::new(hmac::HMAC_SHA256, &key_value);

    // let message = "Hello, world!";
    // let signature = hmac::sign(&key, message.as_bytes());

    // hmac::verify(&key, message.as_bytes(), signature.as_ref())?;

    // Ok(())
}