use josekit::{jws::{JwsHeader, PS256}, jwt::{self, JwtPayload}};
// use josekit::jwt::header;

// use ::jwt::header;
// use rsa::{RsaPrivateKey, RsaPublicKey};
// use rsa::pkcs8::{EncodePrivateKey, DecodePrivateKey, EncodePublicKey, DecodePublicKey, LineEnding};

// const PRIVATE_KEY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "private.pem");

// const PUBLIC_KEY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "public.pem");

const PRIVATE_KEY: &str = "private.pem";
const PUBLIC_KEY: &str = "public.pem";

fn main(){
    // // Generating RSA key pair by openssl
    // let rsa = Rsa::generate(2048).unwrap();
    // let private_key_pem = rsa.private_key_to_pem().unwrap();
    // let public_key_pem = rsa.public_key_to_pem().unwrap();

    // println!("private_key_pem: {:?}", private_key_pem);
    // println!("public_key_pem: {:?}", public_key_pem);


    // // Generating RSA key pair
    // let mut rng = rand::thread_rng();
    // let bits = 2048;
    
    // let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    // let public_key = RsaPublicKey::from(&private_key);

    // // Encoding RSA key pair
    // let private_key_pem = private_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    // let public_key_pem = public_key.to_public_key_pem(LineEnding::LF).unwrap();

    // jwt
    let mut header = JwsHeader::new();
    header.set_token_type("JWT");

    println!("header: {:?}", header);

    let mut payload = JwtPayload::new();
    payload.set_subject("subject");
    println!("payload: {:?}", payload);

    // Signing JWT
    let private_key_pem = std::fs::read(PRIVATE_KEY).unwrap();
    let signer = PS256.signer_from_pem(&private_key_pem).unwrap();
    let jwt = jwt::encode_with_signer(&payload, &header, &signer).unwrap();

    println!("jwt: {:?}", jwt);

    // Verifying JWT
    let public_key_pem = std::fs::read(PUBLIC_KEY).unwrap();
    let verifier = PS256.verifier_from_pem(&public_key_pem).unwrap();
    let (payload, header) = jwt::decode_with_verifier(&jwt, &verifier).unwrap();

    println!("payload: {:?}, header: {:?}", payload, header);


}