use crate::crypto::common::EncryptedValue;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, BlockEncrypt, BlockSizeUser};
use aes::Aes256;
use argon2::{self, hash_raw, verify_raw, Config};
use crypto_box::aead::KeyInit;
use std::error::Error;

use base64::{engine::general_purpose::URL_SAFE, Engine};
use rand_core::{OsRng, RngCore};
pub fn hash(message: &str, salt: &str) -> Result<EncryptedValue, Box<dyn Error>> {
    let config = Config::default();
    let salt = match URL_SAFE.decode(salt) {
        Ok(salt) => salt,
        Err(e) => return Err(Box::new(e)),
    };

    match hash_raw(message.as_bytes(), salt.as_slice(), &config) {
        Ok(hash) => Ok(EncryptedValue {
            cipher: URL_SAFE.encode(hash),
            nonce: URL_SAFE.encode(salt),
        }),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn verify_password(password: &str, hash: &str, salt: &str) -> Result<(), Box<dyn Error>> {
    let hash = match URL_SAFE.decode(hash) {
        Ok(hash) => hash,
        Err(e) => return Err(Box::new(e)),
    };

    let salt = match URL_SAFE.decode(salt) {
        Ok(salt) => salt,
        Err(e) => return Err(Box::new(e)),
    };

    let config = Config::default();
    match verify_raw(
        password.as_bytes(),
        salt.as_slice(),
        hash.as_slice(),
        &config,
    ) {
        Ok(valid) => {
            if valid {
                Ok(())
            } else {
                Err("Incorrect password".into())
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}

const SALT_BYTES: usize = 16;
pub fn generate_salt() -> Result<String, Box<dyn Error>> {
    let mut salt: [u8; SALT_BYTES] = [0; SALT_BYTES];
    OsRng.fill_bytes(&mut salt);
    Ok(URL_SAFE.encode(salt.as_slice()))
}

pub fn encrypt_data(key: &str, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = URL_SAFE.decode(key)?;
    let aes = match Aes256::new_from_slice(key.as_slice()) {
        Ok(aes) => aes,
        Err(_) => return Err("Invalid key".into()),
    };

    let block_size = Aes256::block_size();
    let data = data.to_vec();

    let mut enc: Vec<u8> = Vec::new();

    data.chunks_exact(block_size)
        .enumerate()
        .for_each(|(_, chunk)| {
            let mut chunk = *GenericArray::from_slice(chunk);
            aes.encrypt_block(&mut chunk);
            enc.append(&mut chunk.as_slice().to_vec());
        });

    Ok(enc)
}

pub fn decrypt_data(key: &str, enc: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = URL_SAFE.decode(key)?;
    let aes = match Aes256::new_from_slice(key.as_slice()) {
        Ok(aes) => aes,
        Err(_) => return Err("Invalid key".into()),
    };

    let block_size = Aes256::block_size();

    let mut dec: Vec<u8> = Vec::new();

    enc.chunks_exact(block_size)
        .enumerate()
        .for_each(|(_, chunk)| {
            let mut chunk = *GenericArray::from_slice(chunk);
            aes.decrypt_block(&mut chunk);
            dec.append(&mut chunk.as_slice().to_vec());
        });

    Ok(dec)
}
