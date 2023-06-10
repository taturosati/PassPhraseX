use std::error::Error;
use argon2::{self, Config, hash_raw, verify_raw};
use crate::crypto::common::EncryptedValue;
use base64::{Engine, engine::general_purpose::STANDARD};
use rand_core::{OsRng, RngCore};

pub fn hash_password(password: &str, salt: &str) -> Result<EncryptedValue, Box<dyn Error>> {
    let config = Config::default();
    let salt = match STANDARD.decode(salt) {
        Ok(salt) => salt,
        Err(e) => return Err(Box::new(e)),
    };

    match hash_raw(password.as_bytes(), salt.as_slice(), &config) {
        Ok(hash) => Ok(EncryptedValue {
            cipher: STANDARD.encode(hash),
            nonce: STANDARD.encode(salt),
        }),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn verify_password(password: &str, hash: &str, salt: &str) -> Result<(), Box<dyn Error>> {
    let hash = match STANDARD.decode(hash) {
        Ok(hash) => hash,
        Err(e) => return Err(Box::new(e)),
    };

    let salt = match STANDARD.decode(salt) {
        Ok(salt) => salt,
        Err(e) => return Err(Box::new(e)),
    };

    let config = Config::default();
    match verify_raw(password.as_bytes(), salt.as_slice(),hash.as_slice(), &config) {
        Ok(valid) => {
            if valid {
                Ok(())
            } else {
                Err("Incorrect password".into())
            }
        },
        Err(e) => Err(Box::new(e)),
    }
}

const SALT_BYTES: usize = 16;
pub fn generate_salt() -> Result<String, Box<dyn Error>> {
    let mut salt: [u8; SALT_BYTES] = [0; SALT_BYTES];
    OsRng.fill_bytes(&mut salt);
    Ok(STANDARD.encode(salt.as_slice()))
}