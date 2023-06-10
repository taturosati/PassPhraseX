use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use app_dirs2::{app_dir, AppDataType};
use common::crypto::common::EncryptedValue;
use common::crypto::symmetric::{decrypt_data, encrypt_data};
use crate::{APP_INFO, CredentialsMap};

const DATA_DIR: &str = "data";
const PASSWORD_HASH_FILE: &str = "device_pass";
const PRIVATE_KEY_FILE: &str = "private_key";
const DATA_FILE: &str = "data.json";

fn write_bytes(file_name: &str, bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
	let path_to_file = app_dir(AppDataType::UserData, &APP_INFO, DATA_DIR)?
		.join(file_name);

	let mut file = File::create(path_to_file)?;
	file.write_all(&bytes)?;

	Ok(())
}

fn read_bytes(file_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
	let path_to_file = app_dir(AppDataType::UserData, &APP_INFO, DATA_DIR)?
		.join(file_name);

	let mut file = File::open(path_to_file)?;
	let mut bytes = Vec::new();
	file.read_to_end(&mut bytes)?;

	Ok(bytes)
}

pub fn write_password_hash(hash: &EncryptedValue) -> Result<(), Box<dyn Error>> {
	write_bytes(PASSWORD_HASH_FILE, hash.to_string().as_bytes().to_vec())
}

pub fn read_password_hash() -> Result<EncryptedValue, Box<dyn Error>> {
	let bytes = read_bytes(PASSWORD_HASH_FILE)?;
	Ok(EncryptedValue::from(String::from_utf8(bytes)?))
}

pub fn write_sk(sk: &[u8;32], device_pass_hash: &str) -> Result<(), Box<dyn Error>> {
	let enc = encrypt_data(device_pass_hash, sk)?;
	write_bytes(PRIVATE_KEY_FILE, enc)
}

pub fn read_sk(device_pass_hash: &str) -> Result<[u8;32], Box<dyn Error>> {
	let bytes = read_bytes(PRIVATE_KEY_FILE)?;

	let dec = decrypt_data(device_pass_hash, bytes)?;

	let mut content: [u8;32] = [0;32];
	content.copy_from_slice(&dec[..32]);
	Ok(content)
}

pub fn write_app_data(data: &CredentialsMap) -> Result<(), Box<dyn Error>> {
	let mut string_map = HashMap::new();

	for (site, site_data) in data {
		let mut string_site_data: HashMap<String, String> = HashMap::new();
		for (username, password) in site_data {
			string_site_data.insert(
				username.clone().into(),
				password.clone().into()
			);
		}
		string_map.insert(site.clone(), string_site_data);
	}

	write_bytes(DATA_FILE, serde_json::to_string(&string_map)?.as_bytes().to_vec())
}

pub fn read_app_data() -> Result<CredentialsMap, Box<dyn Error>> {
	let bytes = read_bytes(DATA_FILE)?;
	let data: HashMap<String, HashMap<String, String>> = serde_json::from_slice(&bytes)?;

	// Map data into CredentialsMap
	let mut credentials: CredentialsMap = HashMap::new();

	for (site, site_data) in data {
		let mut enc_site_data: HashMap<EncryptedValue, EncryptedValue> = HashMap::new();
		for (username, password) in site_data {
			enc_site_data.insert(
				EncryptedValue::from(username),
				EncryptedValue::from(password)
			);
		}
		credentials.insert(site.clone(), enc_site_data);
	}

	Ok(credentials)
}
