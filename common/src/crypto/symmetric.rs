use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, BlockEncrypt, BlockSizeUser};
use aes::Aes256;
use anyhow::format_err;
use argon2::{self, verify_raw, Config};
use crypto_box::aead::KeyInit;

use base64::{engine::general_purpose::URL_SAFE, Engine};
use rand_core::{OsRng, RngCore};

pub fn verify_password(password: &str, hash: &str, salt: &str) -> anyhow::Result<()> {
    let hash = URL_SAFE.decode(hash)?;

    let salt = URL_SAFE.decode(salt)?;

    let config = Config::default();
    let valid = verify_raw(
        password.as_bytes(),
        salt.as_slice(),
        hash.as_slice(),
        &config,
    )?;

    if valid {
        Ok(())
    } else {
        Err(format_err!("Invalid password"))
    }
}

const SALT_BYTES: usize = 16;
pub fn generate_salt() -> anyhow::Result<String> {
    let mut salt: [u8; SALT_BYTES] = [0; SALT_BYTES];
    OsRng.fill_bytes(&mut salt);
    Ok(URL_SAFE.encode(salt.as_slice()))
}

pub fn encrypt_data(key: &str, data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let key = URL_SAFE.decode(key)?;
    let aes = match Aes256::new_from_slice(key.as_slice()) {
        Ok(aes) => aes,
        Err(_) => return Err(format_err!("Invalid key")),
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

pub fn decrypt_data(key: &str, enc: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let key = URL_SAFE.decode(key)?;
    let aes = match Aes256::new_from_slice(key.as_slice()) {
        Ok(aes) => aes,
        Err(_) => return Err(format_err!("Invalid key")),
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
