use jws::compact::{decode_verify, encode_sign};
use jws::hmac::{HmacVerifier, Hs512Signer};
use jws::{JsonObject, JsonValue};
// use rsa::pss
// use serde::de;

fn main() {
    // Add custom header parameters.
    let mut header = JsonObject::new();
    header.insert(String::from("typ"), JsonValue::from("text/plain"));

    // Encode and sign the message.
    let encoded = encode_sign(header, "寿限無寿限無".as_bytes(), &Hs512Signer::new(b"secretkey")).unwrap();

    // println!("{:?}", &encoded.payload());
    // println!("{:?}", &encoded.signature());
    println!("encoded_data: {:?}", &encoded.data());

    let encoded_data_as_bytes = encoded.data().as_bytes();

    let serialized = serde_json::to_string(encoded.data()).unwrap();

    // println!("serialized: {:?}", &serialized);

    let deserialized: JsonValue = serde_json::from_str(&serialized).unwrap();


    // println!("deserialized: {:?}", &deserialized.as_str().unwrap());

    // println!("encoded_data_as_bytes: {:?}", &encoded_data_as_bytes);
    // println!("deserialized: {:?}", &deserialized.as_str().unwrap().as_bytes());

    assert_eq!(encoded_data_as_bytes, deserialized.as_str().unwrap().as_bytes());


    // Decode and verify the message.
    let decoded = decode_verify(deserialized.as_str().unwrap().as_bytes(), &HmacVerifier::new(b"secretkey")).unwrap();
    println!("{:?}",     String::from_utf8_lossy(&decoded.payload));



    // decoded.payload;


    // assert_eq!(decoded.payload, b"payload");
    // assert_eq!(
    //     decoded.header.get("typ").and_then(|x| x.as_str()),
    //     Some("text/plain")
    // );
}
