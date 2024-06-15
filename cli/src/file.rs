use crate::{CredentialsMap, APP_INFO};
use app_dirs2::{app_dir, AppDataType};
use std::fs::{File};
use std::io::{Read, Write};

const DATA_DIR: &str = "data";
const PRIVATE_KEY_FILE: &str = "private_key";
const DATA_FILE: &str = "data.json";

fn write_bytes(file_name: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
    let path_to_file = app_dir(AppDataType::UserData, &APP_INFO, DATA_DIR)?.join(file_name);

    let mut file = File::create(path_to_file)?;
    file.write_all(&bytes)?;

    Ok(())
}

fn read_bytes(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path_to_file = app_dir(AppDataType::UserData, &APP_INFO, DATA_DIR)?.join(file_name);

    let mut file = File::open(path_to_file)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    Ok(bytes)
}

pub fn write_sk(sk: Vec<u8>) -> anyhow::Result<()> {
    write_bytes(PRIVATE_KEY_FILE, sk)
}

pub fn read_sk() -> anyhow::Result<Vec<u8>> {
    read_bytes(PRIVATE_KEY_FILE)
}

pub fn write_app_data(data: &CredentialsMap) -> anyhow::Result<()> {
    write_bytes(DATA_FILE, serde_json::to_string(&data)?.as_bytes().to_vec())
}

pub fn read_app_data() -> anyhow::Result<CredentialsMap> {
    let bytes = read_bytes(DATA_FILE)?;
    let data: CredentialsMap = serde_json::from_slice(&bytes)?;
    Ok(data)
}
