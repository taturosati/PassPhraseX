mod file;

use anyhow::format_err;
use passphrasex_common::api::Api;
use std::collections::HashMap;
use std::string::String;

use app_dirs2::AppInfo;

use crate::file::{read_app_data, read_private_key, read_signing_key, write_app_data, write_private_key, write_signing_key};

use passphrasex_common::crypto::asymmetric::{KeyPair, SeedPhrase};
use passphrasex_common::model::password::Password;

pub const APP_INFO: AppInfo = AppInfo {
    name: "PassPhraseX",
    author: "Santos Matías Rosati",
};

// Map of site -> Map of username -> password
pub type CredentialsMap = HashMap<String, HashMap<String, Password>>;

pub struct App {
    key_pair: KeyPair,
    credentials: CredentialsMap,
    api: Api,
}

pub async fn register(device_pass: &str) -> anyhow::Result<SeedPhrase> {
    let seed_phrase = SeedPhrase::new();
    let key_pair = KeyPair::new(seed_phrase.clone());

    let api = Api::new(key_pair.clone());

    write_private_key(key_pair.get_private_key_enc(device_pass))?;
    write_signing_key(key_pair.get_signing_key_enc(device_pass))?;

    write_app_data(&HashMap::new())?;

    api.create_user(key_pair.get_verifying_key()).await?;

    Ok(seed_phrase)
}

pub async fn auth_device(seed_phrase: &str, device_pass: &str) -> anyhow::Result<()> {
    let seed_phrase = SeedPhrase::from(seed_phrase.to_string());
    let key_pair = KeyPair::new(seed_phrase.clone());

    let api = Api::new(key_pair.clone());

    write_private_key(key_pair.get_private_key_enc(device_pass))?;
    write_signing_key(key_pair.get_signing_key_enc(device_pass))?;

    sync_with_api(api, key_pair.clone()).await?;

    Ok(())
}

async fn sync_with_api(api: Api, key_pair: KeyPair) -> anyhow::Result<CredentialsMap> {
    let passwords = api.get_passwords(key_pair.get_verifying_key()).await?;
    let mut credentials: CredentialsMap = HashMap::new();

    for password in passwords {
        credentials
            .entry(password.site.clone())
            .or_default()
            .insert(password._id.clone(), password.clone());
    }

    write_app_data(&credentials)?;

    Ok(credentials)
}

impl App {
    pub async fn new(device_pass: &str) -> anyhow::Result<App> {
        let private_key_enc = read_private_key()?;
        let signing_key_enc = read_signing_key()?;

        let key_pair = KeyPair::try_from_private_keys(
            private_key_enc.as_slice(),
            signing_key_enc.as_slice(),
            device_pass
        )?;

        let api = Api::new(key_pair.clone());

        let credentials = sync_with_api(api, key_pair.clone()).await.or_else(|_| {
            println!("Failed to sync with API, using local data");
            read_app_data()
        })?;

        Ok(App {
            key_pair: key_pair.clone(),
            credentials,
            api: Api::new(key_pair),
        })
    }

    pub async fn add(
        &mut self,
        site: String,
        username: String,
        password: String,
    ) -> anyhow::Result<()> {
        self.verify_credentials_dont_exist(&site, &username)?;

        let user_id = self.key_pair.get_verifying_key();

        let password_id = self.key_pair.hash(&format!("{}{}", site, username));

        let password = Password {
            _id: password_id.clone(),
            user_id: user_id.clone(),
            site: site.clone(),
            username,
            password,
        };
        let password = password.encrypt(&self.key_pair);

        self.api.add_password(user_id, password.clone()).await?;

        self.credentials
            .entry(site)
            .or_default()
            .insert(password_id, password);

        write_app_data(&self.credentials).expect("Failed to save app data to file");
        Ok(())
    }

    pub async fn get(
        &mut self,
        site: String,
        username: Option<String>,
    ) -> anyhow::Result<Vec<Password>> {
        match self.credentials.get(&site) {
            Some(passwords) => match username {
                Some(username) => {
                    let id = self.key_pair.hash(&format!("{}{}", site, username));
                    let password = passwords
                        .get(&id)
                        .ok_or(format_err!("Password not found"))?;

                    Ok(vec![password.decrypt(&self.key_pair)])
                }
                None => {
                    let result = passwords
                        .iter()
                        .map(|(_, password)| password.decrypt(&self.key_pair))
                        .collect();

                    Ok(result)
                }
            },
            None => Err(format_err!("No passwords found")),
        }
    }

    pub async fn edit(
        &mut self,
        site: String,
        username: String,
        password: String,
    ) -> anyhow::Result<()> {
        self.verify_credentials_exist(&site, &username)?;

        let user_id = self.key_pair.get_verifying_key();
        let password_id = self.key_pair.hash(&format!("{}{}", site, username));

        let password_enc = self.key_pair.encrypt(&password);
        self.api
            .edit_password(user_id, password_id.clone(), password_enc.clone())
            .await?;

        self.credentials
            .entry(site)
            .or_default() // Should never happen
            .entry(password_id)
            .and_modify(|e| e.password.clone_from(&password_enc));

        write_app_data(&self.credentials).expect("Failed to save app data to file");

        Ok(())
    }

    pub async fn delete(&mut self, site: String, username: String) -> anyhow::Result<()> {
        self.verify_credentials_exist(&site, &username)?;

        let user_id = self.key_pair.get_verifying_key();
        let password_id = self.key_pair.hash(&format!("{}{}", site, username));

        self.api
            .delete_password(user_id, password_id.clone())
            .await?;

        self.credentials
            .entry(site)
            .or_default() // Should never happen
            .remove(&password_id);

        write_app_data(&self.credentials).expect("Failed to save app data to file");

        Ok(())
    }

    fn verify_credentials_exist(&self, site: &str, username: &str) -> anyhow::Result<()> {
        match self.credentials.get(site) {
            Some(passwords) => {
                let id = self.key_pair.hash(&format!("{}{}", site, username));
                passwords
                    .get(&id)
                    .ok_or(format_err!("Credentials not found"))?;
                Ok(())
            }
            None => Err(format_err!("Credentials not found")),
        }
    }

    fn verify_credentials_dont_exist(&self, site: &str, username: &str) -> anyhow::Result<()> {
        match self.verify_credentials_exist(site, username) {
            Ok(_) => Err(format_err!("Credentials already exist")),
            Err(_) => Ok(()),
        }
    }
}
