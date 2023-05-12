use std::hash::Hash;
use std::str;
use base64::{engine::{general_purpose::STANDARD}, Engine};
use bip32::{Mnemonic, XPrv};
use crypto_box::{
    aead::{Aead, AeadCore, OsRng},
    ChaChaBox, PublicKey, SecretKey, Nonce
};
use crypto_box::aead::Payload;

#[derive(Eq, PartialEq)]
pub struct EncryptedValue {
    pub cipher: String,
    pub nonce: Nonce,
}

impl Hash for EncryptedValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cipher.hash(state);
    }
}

// derive clone
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
        // Generate random Mnemonic using the default language (English)
        // let mnemonic = Mnemonic::random(&mut OsRng, Default::default());
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
            cipher: STANDARD.encode(&enc),
            nonce
        }
    }

    pub fn decrypt(&self, enc: &EncryptedValue) -> String {
        let cipher = STANDARD.decode(enc.cipher.as_bytes()).unwrap();
        let personal_box = ChaChaBox::new(&self.public_key, &self.private_key);

        let dec = personal_box.decrypt(&enc.nonce,Payload {
            msg: cipher.as_slice(),
            aad: b"",
        }).unwrap();

        str::from_utf8(&dec).unwrap().to_owned()
    }
}
