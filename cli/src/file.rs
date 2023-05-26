use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use app_dirs2::{app_dir, AppDataType};
use crate::APP_INFO;

const DATA_DIR: &str = "data";
const PRIVATE_KEY_FILE: &str = "private_key";

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

pub fn write_sk(sk: &[u8;32]) -> Result<(), Box<dyn Error>> {
	write_bytes(PRIVATE_KEY_FILE, sk.to_vec())
}

pub fn read_sk(_device_pass: &str) -> Result<[u8;32], Box<dyn Error>> {
	let mut content: [u8;32] = [0;32];
	let bytes = read_bytes(PRIVATE_KEY_FILE)?;
	content.copy_from_slice(&bytes[..32]);
	Ok(content)
}