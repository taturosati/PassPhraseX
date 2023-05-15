use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use app_dirs2::{app_dir, AppDataType, AppInfo};
use common::{KeyPair, SeedPhrase, EncryptedValue};

const APP_INFO: AppInfo = AppInfo{name: "PassPhraseX", author: "Santos Rosati"};

pub struct App<> {
    key_pair: KeyPair,
    // HashMap<site, HashMap<username, password>>
    credentials: HashMap<String, HashMap<String, EncryptedValue>>,
}

pub fn register(device_pass: &str) -> SeedPhrase {
    let seed_phrase = SeedPhrase::new();
    let key_pair = KeyPair::new(seed_phrase.clone());

    // let cipher = key_pair.encrypt("Test message");
    // println!("Cipher: {:?}", cipher.cipher);
    // let decrypted = key_pair.decrypt(&cipher);
    //
    // assert_eq!(decrypted, "Test message");

    let path_to_file = match app_dir(AppDataType::UserData, &APP_INFO, "data") {
        Ok(path) => path.join("private_key"),
        Err(e) => panic!("Error: {}", e)
    };

    println!("Path: {:?}", path_to_file);
    match File::create(path_to_file) {
        Ok(mut file) => {
            match file.write_all(key_pair.private_key.as_bytes()) {
                Ok(_) => seed_phrase,
                Err(e) => panic!("Error: {}", e)
            }
        },
        Err(e) => panic!("Error: {}", e)
    }
}

pub fn auth_device(seed_phrase: &str, device_pass: &str) {
    let seed_phrase = SeedPhrase::from_str(seed_phrase);
    let key_pair = KeyPair::new(seed_phrase.clone());

    let path_to_file = match app_dir(AppDataType::UserData, &APP_INFO, "data") {
        Ok(path) => path.join("private_key"),
        Err(e) => panic!("Error: {}", e)
    };

    match File::create(path_to_file) {
        Ok(mut file) => {
            match file.write_all(key_pair.private_key.as_bytes()) {
                Ok(_) => println!("Successfully authenticated device!"),
                Err(e) => panic!("Error: {}", e)
            }
        },
        Err(e) => panic!("Error: {}", e)
    }
}

impl App {
    pub fn new(device_pass: &str) -> App {
        let private_key = get_sk(device_pass);
        let key_pair = KeyPair::from_sk(private_key);
        App {
            key_pair,
            credentials: HashMap::new()
        }
    }

    pub fn add(&mut self, site: String, username: String, password: String) {
        let mut site_credentials: HashMap<String, EncryptedValue> = HashMap::new();
        site_credentials.insert(
            username,
            self.key_pair.encrypt(&password)
        );

        self.credentials.insert(site, site_credentials);
        // TODO: Write to file
    }

    pub fn get(self, site: String, username: Option<String>) {
        let site_credentials = self.credentials.get(&site).unwrap();
        // TODO: Print all usernames if username is None
        let encrypted_password = site_credentials.get(&username.unwrap()).unwrap();
        let password = self.key_pair.decrypt(encrypted_password);

        println!("{}", password);
    }
}

fn get_sk(device_pass: &str) -> [u8;32] {
    let path_to_file = match app_dir(AppDataType::UserData, &APP_INFO, "data") {
        Ok(path) => path.join("private_key"),
        Err(e) => panic!("Error: {}", e)
    };

    match File::open(path_to_file) {
        Ok(mut file) => {
            let mut content: [u8;32] = [0;32];
            match file.read_exact(&mut content) {
                Ok(_) => content,
                Err(e) => panic!("Error: {}", e)
            }
        },
        Err(e) => panic!("Error: {}", e)
    }
}