/*
* Password Manager
* Stores passwords encrypted via a private - public key pair
*/
mod cryptography;

use std::collections::HashMap;
use clap::{Parser, Subcommand};
use std::fs::File;
use std::string::String;
use std::io::{Read, Write};
use std::ops::Deref;
use app_dirs2::*;
use rust_sodium::crypto::box_::Nonce;
use crate::cryptography::{EncryptedValue, KeyPair, SeedPhrase};
use crypto_box::{
    aead::{Aead, AeadCore, OsRng},
    ChaChaBox, PublicKey, SecretKey
};

const APP_INFO: AppInfo = AppInfo{name: "PassPhraseX", author: "Santos Rosati"};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// A simple password manager
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize the password manager
    Register {
        #[clap(short, long)]
        device_pass: String,
    },
    /// Login to the password manager using your seed phrase
    Login {
        #[clap(short, long)]
        seed_phrase: String,
        #[clap(short, long)]
        device_pass: String,
    },
    /// Add a new password to the password manager
    Add {
        #[clap(short, long)]
        site: String,
        #[clap(short, long)]
        username: String,
        #[clap(short, long)]
        password: String,
        #[clap(short, long)]
        device_pass: String,
    },
    /// Get a password from the password manager
    Get {
        #[clap(short, long)]
        site: String,
        #[clap(short, long)]
        username: Option<String>,
        #[clap(short, long)]
        device_pass: String,
    },
}

struct App {
    key_pair: Option<KeyPair>,

    // HashMap<site, HashMap<username, password>>
    credentials: HashMap<String, HashMap<String, EncryptedValue>>
}

impl App {
    fn new() -> App {
        App { key_pair: None, credentials: HashMap::new() }
    }

    fn init(&mut self, device_pass: &str) {
        let private_key = get_sk(device_pass);
        self.key_pair = Some(cryptography::KeyPair::from_sk(private_key));
        // TODO: Read credentials from file
        self.credentials = HashMap::new();
    }

    fn register(&mut self, device_pass: &str) -> SeedPhrase {
        let seed_phrase = SeedPhrase::new();
        let key_pair = KeyPair::new(seed_phrase.clone());

        let sk = SecretKey::from(key_pair.private_key.0.);

        let path_to_file = match app_dir(AppDataType::UserData, &APP_INFO, "data") {
            Ok(path) => path.join("private_key"),
            Err(e) => panic!("Error: {}", e)
        };

        println!("Path: {:?}", path_to_file);
        match File::create(path_to_file) {
            Ok(mut file) => {
                match file.write_all(&key_pair.private_key[..]) {
                    Ok(_) => seed_phrase,
                    Err(e) => panic!("Error: {}", e)
                }
            },
            Err(e) => panic!("Error: {}", e)
        }
    }

    fn login(&mut self, seed_phrase: &str, device_pass: &str) {
        let seed_phrase = SeedPhrase::from_str(seed_phrase);
        let key_pair = KeyPair::new(seed_phrase.clone());

        let path_to_file = match app_dir(AppDataType::UserData, &APP_INFO, "data") {
            Ok(path) => path.join("private_key"),
            Err(e) => panic!("Error: {}", e)
        };

        println!("Path: {:?}", path_to_file);
        match File::create(path_to_file) {
            Ok(mut file) => {
                match file.write_all(&key_pair.private_key[..]) {
                    Ok(_) => println!("Successfully authenticated on device"),
                    Err(e) => panic!("Error: {}", e)
                }
            },
            Err(e) => panic!("Error: {}", e)
        };
    }

    fn add(&mut self, site: String, username: String, password: String) {
        let mut site_credentials: HashMap<String, EncryptedValue> = HashMap::new();
        site_credentials.insert(
            username,
            self.key_pair.as_ref().expect("Unauthenticated").encrypt(&password)
        );

        self.credentials.insert(site, site_credentials);
        // TODO: Write to file
    }

    fn get(self, site: String, username: Option<String>) {
        let site_credentials = self.credentials.get(&site).unwrap();
        // TODO: Print all usernames if username is None
        let encrypted_password = site_credentials.get(&username.unwrap()).unwrap();
        let password = self.key_pair.expect("Unauthenticated").decrypt(encrypted_password);

        println!("{}", password);
    }
}

fn main() {
    let args = Args::parse();
    let mut app = App::new();

    match args.command {
        Commands::Register {device_pass} => {
            app.register(&device_pass);
        },
        Commands::Login { seed_phrase, device_pass } => {
            app.login(&seed_phrase, &device_pass)
        },
        Commands::Add { site, username, password, device_pass } => {
            app.init(&device_pass);
            app.add(site, username, password);
        },
        Commands::Get { site, username, device_pass } => {
            app.init(&device_pass);
            app.get(site, username);
        }
    }

}

// fn init(device_pass: String) {
//     let key_pair = cryptography::KeyPair::new(None);
//
//     let path_to_file = match app_dir(AppDataType::UserData, &APP_INFO, "data") {
//         Ok(path) => path.join("private_key"),
//         Err(e) => panic!("Error: {}", e)
//     };
//
//     println!("Path: {:?}", path_to_file);
//     match File::create(path_to_file) {
//         Ok(mut file) => {
//             match file.write_all(&key_pair.private_key[..]) {
//                 Ok(_) => println!("Private key saved"),
//                 Err(e) => panic!("Error: {}", e)
//             }
//         },
//         Err(e) => panic!("Error: {}", e)
//     }
// }
//
// fn login(seed_phrase: String, device_pass: String) {
//     println!("Login");
// }
//
// fn add(app_data: AppData, site: String, username: String, password: String) {
//     let username = app_data.key_pair.encrypt(&username);
//     let password = app_data.key_pair.encrypt(&password);
//
//     // print encrypted username and password
//     println!("Username: {} {:?}", username.cipher, username.nonce);
//     println!("Password: {} {:?}", password.cipher, password.nonce);
//
//     // TODO: Save to file
// }
//
// fn get(app_data: AppData, site: String, username: Option<String>) {
//     // TODO: Read from file
//     let site_credentials = app_data.credentials.get(&site).unwrap();
//     let password = app_data.credentials.get(&site).unwrap().get(&app_data.key_pair.encrypt(&username.unwrap())).unwrap();
//     println!("Get");
// }

fn get_sk(device_pass: &str) -> Vec<u8> {
    let path_to_file = match app_dir(AppDataType::UserData, &APP_INFO, "data") {
        Ok(path) => path.join("private_key"),
        Err(e) => panic!("Error: {}", e)
    };

    match File::open(path_to_file) {
        Ok(mut file) => {
            let mut content = Vec::new();
            match file.read_to_end(&mut content) {
                Ok(_) => content,
                Err(e) => panic!("Error: {}", e)
            }
        },
        Err(e) => panic!("Error: {}", e)
    }
}


