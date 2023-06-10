use std::str;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use bip32::{Mnemonic, XPrv};
use crypto_box::aead::{Aead, AeadCore, OsRng, Payload};
use crypto_box::{ChaChaBox, Nonce, PublicKey, SecretKey};
use crate::crypto::common::EncryptedValue;

#[derive(Clone)]
pub struct SeedPhrase {
    phrase: String
}

impl SeedPhrase {
    pub fn new() -> SeedPhrase {
        let mnemonic = Mnemonic::random(&mut OsRng, Default::default());
        SeedPhrase { phrase: mnemonic.phrase().to_owned() }
    }

    pub fn get_phrase(&self) -> String {
        self.phrase.clone()
    }

    pub fn from_str(phrase: &str) -> SeedPhrase {
        SeedPhrase { phrase: phrase.to_owned() }
    }
}

pub struct KeyPair {
    pub private_key: SecretKey,
    pub public_key: PublicKey,
}

/*
* Implement asymmetric encryption functions for struct KeyPair
* 2 methods -> encrypt & decrypt
*/
impl KeyPair {
    pub fn new(seed_phrase: SeedPhrase) -> KeyPair {
        // Get Mnemonic using the default language (English)
        let mnemonic = Mnemonic::new(seed_phrase.get_phrase(), Default::default()).unwrap();

        // Derive a BIP39 seed value using the given password
        let seed = mnemonic.to_seed("");

        // Derive the root `XPrv` from the `seed` value
        let derived_sk = XPrv::new(&seed).unwrap();

        // Convert the `XPrv` to a `SecretKey` and `PublicKey`
        let private_key = SecretKey::from(derived_sk.to_bytes());
        let public_key = private_key.public_key();

        KeyPair { private_key, public_key }
    }

    pub fn from_sk(sk: [u8;32]) -> KeyPair {
        let private_key = SecretKey::from(sk);
        let public_key = private_key.public_key();
        KeyPair { private_key, public_key }
    }

    pub fn encrypt(&self, message: &str) -> EncryptedValue {
        let nonce = ChaChaBox::generate_nonce(&mut OsRng);

        let personal_box = ChaChaBox::new(&self.public_key, &self.private_key);
        let enc = personal_box.encrypt(&nonce,Payload {
            msg: message.as_bytes(),
            aad: b"",
        }).unwrap();
        EncryptedValue {
            cipher: URL_SAFE.encode(&enc),
            nonce: URL_SAFE.encode(nonce)
        }
    }

    pub fn decrypt(&self, enc: &EncryptedValue) -> String {
        let cipher = URL_SAFE.decode(enc.cipher.as_bytes()).unwrap();
        let personal_box = ChaChaBox::new(&self.public_key, &self.private_key);

        let nonce = URL_SAFE.decode(enc.nonce.as_bytes()).expect("Failed to decode nonce");
        let mut content: [u8;24] = [0;24];
        content.copy_from_slice(&nonce);
        let dec = personal_box.decrypt(&Nonce::from(content),Payload {
            msg: cipher.as_slice(),
            aad: b"",
        }).unwrap();

        str::from_utf8(&dec).unwrap().to_owned()
    }

    pub fn get_pk(&self) -> String {
        URL_SAFE.encode(&self.public_key)
    }
}
