mod api;
mod file;

use std::collections::HashMap;
use std::error::Error;
use std::string::String;

use app_dirs2::AppInfo;

use api::Api;
use common::crypto::asymmetric::{KeyPair, SeedPhrase};
use common::crypto::common::{EncryptedValue};
use common::crypto::symmetric::{encrypt_data, generate_salt, hash, verify_password};
use common::model::password::Password;
use crate::file::{read_app_data, read_sk, write_app_data, write_sk, write_password_hash, read_password_hash};

pub const APP_INFO: AppInfo = AppInfo{name: "PassPhraseX", author: "Santos Matías Rosati"};

// Map of site -> Map of username -> password
pub type CredentialsMap = HashMap<String, HashMap<String, Password>>;

pub struct App<> {
    key_pair: KeyPair,
    credentials: CredentialsMap,
    api: Api
}

pub async fn register(device_pass: &str) -> Result<SeedPhrase, Box<dyn Error>> {
    let salt = generate_salt()?;
    let pass_hash = hash(device_pass, &salt)?;

    let seed_phrase = SeedPhrase::new();
    let key_pair = KeyPair::new(seed_phrase.clone());

    let api = Api::new(key_pair.clone());

    write_password_hash(&pass_hash)?;

    let enc = encrypt_data(&pass_hash.cipher, key_pair.private_key.as_bytes())?;

    let mut sk_bytes:[u8; 32] = [0;32];
    sk_bytes.copy_from_slice(&enc.as_slice());
    write_sk(key_pair.private_key.as_bytes(), &pass_hash.cipher)?;

    write_app_data(&HashMap::new())?;

    api.create_user(key_pair.get_pk()).await?;

    Ok(seed_phrase)
}

pub async fn auth_device(seed_phrase: &str, device_pass: &str) -> Result<(), Box<dyn Error>> {
    let salt = generate_salt()?;
    let pass_hash = hash(device_pass, &salt)?;

    let seed_phrase = SeedPhrase::from_str(seed_phrase);
    let key_pair = KeyPair::new(seed_phrase.clone());

    let api = Api::new(key_pair.clone());

    write_password_hash(&pass_hash)?;

    write_sk(key_pair.private_key.as_bytes(), &pass_hash.cipher)?;

    sync_with_api(api, key_pair.clone()).await?;

    Ok(())
}

async fn sync_with_api(api: Api, key_pair: KeyPair) -> Result<CredentialsMap, Box<dyn Error>> {
    let passwords = api.get_passwords(key_pair.get_pk()).await?;
    let mut credentials: CredentialsMap = HashMap::new();

    for password in passwords {
        credentials.entry(password.site.clone())
            .or_insert(HashMap::new())
            .insert(password._id.clone(), password.clone());
    };

    write_app_data(&credentials)?;

    Ok(credentials)
}

impl App {
    pub async fn new(device_pass: &str) -> Result<App, Box<dyn Error>> {
        let pass_hash = read_password_hash()?;
        verify_password(device_pass, &pass_hash.cipher, &pass_hash.nonce)?;

        let private_key = read_sk(&pass_hash.cipher)?;
        let key_pair = KeyPair::from_sk(private_key);

        let api = Api::new(key_pair.clone());

        let credentials = sync_with_api(api, key_pair.clone()).await.or_else(|_| {
            println!("Failed to sync with API, using local data");
            read_app_data()
        })?;

        Ok(App {
            key_pair: key_pair.clone(),
            credentials,
            api: Api::new(key_pair)
        })
    }

    pub async fn add(&mut self, site: String, username: String, password: String) -> Result<(), Box<dyn Error>> {
        self.verify_credentials_dont_exist(&site, &username)?;

        let public_key = self.key_pair.get_pk();
        let username_enc = self.key_pair.encrypt(&username);
        let password_enc = self.key_pair.encrypt(&password);

        let site_username_hash = self.key_pair.hash(&format!("{}{}", site, username))?;

        let password = Password {
            _id: site_username_hash.clone(),
            site: site.clone(),
            username: username_enc.clone().into(),
            password: password_enc.clone().into()
        };

        self.api.add_password(
            public_key,
            password.clone()
        ).await?;

        self.credentials.entry(site)
            .or_insert(HashMap::new())
            .insert(site_username_hash, password);

        write_app_data(&self.credentials).expect("Failed to save app data to file");
        Ok(())
    }

    pub async fn get(&mut self, site: String, username: Option<String>) -> Result<Vec<Password>, Box<dyn Error>> {
        match self.credentials.get(&site) {
            Some(passwords) => {
                match username {
                    Some(username) => {
                        let id = self.key_pair.hash(&format!("{}{}", site, username))?;
                        let password = passwords.get(&id).ok_or("Password not found")?;

                        let username_dec = self.key_pair.decrypt(
                            &EncryptedValue::from(password.username.clone()));
                        let password_dec = self.key_pair.decrypt(
                            &EncryptedValue::from(password.password.clone()));

                        let password = Password {
                            _id: id.to_string(),
                            site: site.clone(),
                            username: username_dec,
                            password: password_dec
                        };

                        Ok(vec![password.clone()])
                    },
                    None => {
                        let mut result: Vec<Password> = Vec::new();
                        for (id, password) in passwords {
                            let username_dec = self.key_pair.decrypt(&EncryptedValue::from(password.username.clone()));
                            let password_dec = self.key_pair.decrypt(&EncryptedValue::from(password.password.clone()));

                            result.push(Password {
                                _id: id.to_string(),
                                site: site.clone(),
                                username: username_dec,
                                password: password_dec
                            });
                        }

                        Ok(result)
                    }
                }
            },
            None => {
                Err("No passwords found".into())
            }
        }
    }

    pub async fn edit(&mut self, site: String, username: String, password: String) -> Result<(), Box<dyn Error>> {
        self.verify_credentials_exist(&site, &username)?;

        let public_key = self.key_pair.get_pk();
        let site_username_hash = self.key_pair.hash(&format!("{}{}", site, username))?;

        let password_enc = self.key_pair.encrypt(&password);
        self.api.edit_password(
            public_key,
            site_username_hash.clone(),
            password_enc.clone().into()
        ).await?;

        self.credentials.entry(site)
            .or_insert(HashMap::new()) // Should never happen
            .entry(site_username_hash)
            .and_modify(|e| e.password = password_enc.clone().into());

        write_app_data(&self.credentials).expect("Failed to save app data to file");

        Ok(())
    }

    pub async fn delete(&mut self, site: String, username: String) -> Result<(), Box<dyn Error>> {
        self.verify_credentials_exist(&site, &username)?;

        let public_key = self.key_pair.get_pk();
        let site_username_hash = self.key_pair.hash(&format!("{}{}", site, username))?;

        self.api.delete_password(
            public_key,
            site_username_hash.clone()
        ).await?;

        self.credentials.entry(site)
            .or_insert(HashMap::new()) // Should never happen
            .remove(&site_username_hash);

        write_app_data(&self.credentials).expect("Failed to save app data to file");

        Ok(())
    }

    fn verify_credentials_exist(&self, site: &str, username: &str) -> Result<(), Box<dyn Error>> {
        match self.credentials.get(site) {
            Some(passwords) => {
                let id = self.key_pair.hash(&format!("{}{}", site, username))?;
                passwords.get(&id).ok_or("Credentials not found")?;
                Ok(())
            },
            None => {
                Err("Credentials not found".into())
            }
        }
    }

    fn verify_credentials_dont_exist(&self, site: &str, username: &str) -> Result<(), Box<dyn Error>> {
        match self.verify_credentials_exist(site, username) {
            Ok(_) => Err("Credentials already exist".into()),
            Err(_) => Ok(())
        }
    }
}