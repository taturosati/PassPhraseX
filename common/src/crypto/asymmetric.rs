use base64::{engine::general_purpose::URL_SAFE, Engine};
use bip32::secp256k1::ecdsa::signature::Keypair;
use bip32::secp256k1::sha2::Sha256;
use bip32::{ChildNumber, Mnemonic, XPrv};
use crypto_box::aead::OsRng;
use crypto_box::PublicKey;
use pkcs8::EncodePrivateKey;
use rand::thread_rng;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey};
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier};
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::str;

#[derive(Clone)]
pub struct SeedPhrase {
    phrase: String,
}

impl SeedPhrase {
    pub fn new() -> SeedPhrase {
        let mnemonic = Mnemonic::random(OsRng, Default::default());
        SeedPhrase {
            phrase: mnemonic.phrase().to_owned(),
        }
    }

    pub fn get_phrase(&self) -> String {
        self.phrase.clone()
    }
}

impl Default for SeedPhrase {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for SeedPhrase {
    fn from(value: String) -> SeedPhrase {
        SeedPhrase { phrase: value }
    }
}

#[derive(Clone)]
pub struct KeyPair {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
    pub signing_key: SigningKey<Sha256>,
    pub verifying_key: VerifyingKey<Sha256>,
}

/*
* Implement asymmetric encryption functions for struct KeyPair
* 2 methods -> encrypt & decrypt
*/
impl KeyPair {
    pub fn new(seed_phrase: SeedPhrase) -> KeyPair {
        Self::try_new(seed_phrase).expect("Failed to create key pair")
    }

    pub fn try_new(seed_phrase: SeedPhrase) -> anyhow::Result<KeyPair> {
        // Get Mnemonic using the default language (English)
        let mnemonic = Mnemonic::new(seed_phrase.get_phrase(), Default::default())
            .map_err(|_| anyhow::format_err!("Failed to create mnemonic"))?;

        // Derive a BIP39 seed value using the given password
        let seed = mnemonic.to_seed("");

        // Derive the root `XPrv` from the `seed` value
        let derived_key =
            XPrv::new(&seed).map_err(|_| anyhow::format_err!("Failed to derive sk"))?;

        let derived_signing_key = derived_key
            .derive_child(ChildNumber(1))
            .map_err(|_| anyhow::format_err!("Failed to derive signing sk"))?;

        let priv_attrs = derived_key.attrs();
        let chain_code: [u8; 32] = priv_attrs.chain_code;
        let mut seed = ChaCha20Rng::from_seed(chain_code);
        let private_key = RsaPrivateKey::new(&mut seed, 2048)
            .map_err(|_| anyhow::format_err!("Failed to create private key"))?;

        let public_key = RsaPublicKey::from(&private_key);

        let chain_code = derived_signing_key.attrs().chain_code;
        seed = ChaCha20Rng::from_seed(chain_code);
        let signing_private_key = RsaPrivateKey::new(&mut seed, 2048)
            .map_err(|_| anyhow::format_err!("Failed to create signing private key"))?;

        let signing_key = SigningKey::from(signing_private_key.clone());
        let verifying_key = VerifyingKey::from(RsaPublicKey::from(&signing_private_key));

        Ok(KeyPair {
            private_key,
            public_key,
            signing_key,
            verifying_key,
        })
    }

    pub fn try_from_private_keys(
        private_key: &[u8],
        signing_key: &[u8],
        password: &str,
    ) -> anyhow::Result<KeyPair> {
        let private_key = RsaPrivateKey::from_pkcs8_encrypted_der(private_key, password)
            .map_err(|_| anyhow::format_err!("Invalid device password"))?;

        let public_key = private_key.to_public_key();

        let signing_private_key = RsaPrivateKey::from_pkcs8_encrypted_der(signing_key, password)
            .map_err(|_| anyhow::format_err!("Failed to create signing key"))?;

        let signing_key = SigningKey::from(signing_private_key);
        let verifying_key = signing_key.verifying_key();

        Ok(KeyPair {
            private_key,
            public_key,
            signing_key,
            verifying_key,
        })
    }

    pub fn encrypt(&self, message: &str) -> String {
        let enc = self
            .public_key
            .encrypt(&mut thread_rng(), rsa::Pkcs1v15Encrypt, message.as_bytes())
            .expect("Failed to encrypt");

        URL_SAFE.encode(enc)
    }

    pub fn decrypt(&self, enc: &String) -> String {
        let cipher = URL_SAFE.decode(enc.as_bytes()).expect("Failed to decode");
        let dec = self
            .private_key
            .decrypt(rsa::Pkcs1v15Encrypt, &cipher)
            .expect("Failed to decrypt");

        String::from_utf8(dec).unwrap()
    }

    pub fn sign(&self, message: &str) -> Vec<u8> {
        let signature = self
            .signing_key
            .sign_with_rng(&mut thread_rng(), message.as_bytes());
        signature.to_vec()
    }

    pub fn get_public_key(&self) -> String {
        let pk_bytes = self
            .public_key
            .to_pkcs1_der()
            .expect("Failed to convert to pkcs1 der");

        URL_SAFE.encode(pk_bytes.as_bytes())
    }

    pub fn get_private_key_enc(&self, password: &str) -> Vec<u8> {
        let enc = self
            .private_key
            .to_pkcs8_encrypted_der(OsRng, password)
            .expect("Failed to convert to pkcs8 pem");

        enc.to_bytes().to_vec()
    }

    pub fn get_signing_key_enc(&self, password: &str) -> Vec<u8> {
        let enc = self
            .signing_key
            .to_pkcs8_encrypted_der(OsRng, password)
            .expect("Failed to convert to pkcs8 pem");

        enc.to_bytes().to_vec()
    }

    pub fn get_verifying_key(&self) -> String {
        let pk_bytes = self
            .verifying_key
            .to_pkcs1_der()
            .expect("Failed to convert to pkcs1 der");

        URL_SAFE.encode(pk_bytes.as_bytes())
    }

    pub fn hash(&self, message: &str) -> String {
        let signature = self
            .signing_key
            .sign_with_rng(&mut thread_rng(), message.as_bytes());

        signature.to_string()
    }
}

pub fn verifying_key_from_base64(vk: &str) -> anyhow::Result<VerifyingKey<Sha256>> {
    let key_bytes = URL_SAFE
        .decode(vk.as_bytes())
        .map_err(|_| anyhow::format_err!("Failed to decode"))?;

    let public_key = RsaPublicKey::from_pkcs1_der(key_bytes.as_slice())
        .map_err(|_| anyhow::format_err!("Failed to create public key"))?;

    Ok(VerifyingKey::from(public_key))
}

pub fn public_key_from_base64(pk: &str) -> PublicKey {
    let pk_bytes = URL_SAFE.decode(pk.as_bytes()).unwrap();
    let mut buff: [u8; 32] = [0; 32];
    buff.copy_from_slice(pk_bytes.as_slice());
    PublicKey::from(buff)
}

pub fn verify(
    verifying_key: VerifyingKey<Sha256>,
    data: &[u8],
    signature: &[u8],
) -> anyhow::Result<()> {
    let signature = Signature::try_from(signature)
        .map_err(|_| anyhow::format_err!("Failed to convert to signature"))?;

    verifying_key
        .verify(data, &signature)
        .map_err(|err| anyhow::format_err!("Failed to verify: {}", err))?;

    Ok(())
}
