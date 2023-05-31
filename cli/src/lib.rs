mod api;
mod file;

use std::collections::HashMap;
use std::error::Error;
use app_dirs2::AppInfo;
use common::{KeyPair, SeedPhrase, EncryptedValue};
use api::Api;
use std::string::String;
use common::model::password::Password;
use crate::file::{read_app_data, read_sk, write_app_data, write_sk};

pub const APP_INFO: AppInfo = AppInfo{name: "PassPhraseX", author: "Santos Matías Rosati"};

// Map of site -> Map of username -> password
pub type CredentialsMap = HashMap<String, HashMap<EncryptedValue, EncryptedValue>>;

pub struct App<> {
    key_pair: KeyPair,
    credentials: CredentialsMap,
    api: Api
}

pub async fn register(_device_pass: &str) -> Result<SeedPhrase, Box<dyn Error>> {
    let seed_phrase = SeedPhrase::new();
    let key_pair = KeyPair::new(seed_phrase.clone());

    let api = Api::new("http://localhost:3000");

    write_sk(key_pair.private_key.as_bytes())?;

    write_app_data(&HashMap::new())?;

    api.create_user(key_pair.get_pk()).await?;

    Ok(seed_phrase)
}

pub async fn auth_device(seed_phrase: &str, _device_pass: &str) -> Result<(), Box<dyn Error>> {
    let seed_phrase = SeedPhrase::from_str(seed_phrase);
    let key_pair = KeyPair::new(seed_phrase.clone());

    let api = Api::new("http://localhost:3000");

    write_sk(key_pair.private_key.as_bytes())?;

    let passwords = api.get_passwords(key_pair.get_pk(), None, None).await?;
    let mut app_data: CredentialsMap = HashMap::new();

    for password in passwords {
        app_data.entry(password.site)
            .or_insert(HashMap::new())
            .insert(password.username.into(), password.password.into());
    }

    write_app_data(&app_data)?;

    Ok(())
}

impl App {
    pub fn new(device_pass: &str) -> App {
        let private_key = read_sk(device_pass)
            .expect("Failed to read private key from file");
        let key_pair = KeyPair::from_sk(private_key);

        let credentials = read_app_data()
            .expect("Failed to read app data from file");

        App {
            key_pair,
            credentials,
            api: Api::new("http://localhost:3000")
        }
    }

    pub async fn add(&mut self, site: String, username: String, password: String) -> Result<(), Box<dyn Error>>{
        let public_key = self.key_pair.get_pk();
        let username_enc = self.key_pair.encrypt(&username);
        let password_enc = self.key_pair.encrypt(&password);

        self.api.add_password(
            public_key,
            site.clone(),
            username_enc.clone().into(),
            password_enc.clone().into()
        ).await?;

        self.credentials.entry(site)
            .or_insert(HashMap::new())
            .insert(username_enc, password_enc);

        write_app_data(&self.credentials).expect("Failed to save app data to file");
        Ok(())
    }

    pub async fn get(&mut self, site: String, username: Option<String>) -> Result<Vec<Password>, Box<dyn Error>> {
        match self.credentials.get(&site) {
            Some(passwords) => {
                let mut result: Vec<Password> = Vec::new();
                for (username_enc, password_enc) in passwords {
                    let username_dec = self.key_pair.decrypt(&username_enc);
                    let password_dec = self.key_pair.decrypt(&password_enc);

                    result.push(Password {
                        site: site.clone(),
                        username: username_dec,
                        password: password_dec
                    });
                }

                return Ok(result);
            },
            None => {}
        };

        let passwords = self.api.get_passwords(self.key_pair.get_pk(), Some(site.clone()), username).await?;

        if passwords.is_empty() {
            return Err("No passwords found".into());
        }

        let mut result: Vec<Password> = Vec::new();


        let credentials = self
            .credentials.entry(site.clone())
            .or_insert(HashMap::new());

        for credential in passwords {
            let password_enc = EncryptedValue::from(credential.password);
            let username_enc = EncryptedValue::from(credential.username);

            credentials.insert(username_enc.clone(), password_enc.clone());

            let password_dec = self.key_pair.decrypt(&password_enc);
            let username_dec = self.key_pair.decrypt(&username_enc);

            result.push(Password {
                site: site.clone(),
                username: username_dec,
                password: password_dec
            });
        };

        Ok(result)
    }
}