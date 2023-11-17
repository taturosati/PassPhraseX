use crate::app::App;
use anyhow::anyhow;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Object;
use passphrasex_common::api::Api;
use passphrasex_common::crypto::asymmetric::{KeyPair, SeedPhrase};
use passphrasex_common::crypto::symmetric::{encrypt_data, generate_salt, hash};
use passphrasex_common::model::password::Password;
use passphrasex_common::model::CredentialsMap;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_extensions_sys::chrome;

pub static STORAGE_KEYS: [&str; 3] = ["public_key", "secret_key", "salt"];
pub static CREDENTIALS_KEYS: [&str; 1] = ["credentials"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageSecretKey {
    pub public_key: Option<String>,
    pub secret_key: Option<String>,
    pub salt: Option<String>,
}

impl TryInto<Object> for StorageSecretKey {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Object, Self::Error> {
        let js_value = JsValue::from_serde(&self)?;
        Ok(Object::from(js_value))
    }
}

impl StorageSecretKey {
    pub fn new(
        public_key: Option<String>,
        secret_key: Option<String>,
        salt: Option<String>,
    ) -> Self {
        Self {
            public_key,
            secret_key,
            salt,
        }
    }

    pub fn generate(device_password: String) -> anyhow::Result<(Self, String, KeyPair)> {
        let salt = generate_salt()?;
        let pass_hash = hash(&device_password, &salt)?;

        let seed_phrase = SeedPhrase::new();
        let key_pair = KeyPair::try_new(seed_phrase.clone())?;

        let enc_sk = encrypt_data(&pass_hash.cipher, key_pair.private_key.as_bytes())?;
        let secret_key = hex::encode(enc_sk.as_slice());

        let public_key = key_pair.get_pk();

        Ok((
            Self::new(Some(public_key), Some(secret_key), Some(salt)),
            seed_phrase.get_phrase(),
            key_pair,
        ))
    }

    pub async fn from_seed_phrase(
        seed_phrase: String,
        device_password: String,
    ) -> anyhow::Result<(Self, KeyPair)> {
        let salt = generate_salt()?;
        let pass_hash = hash(&device_password, &salt)?;

        let seed_phrase = SeedPhrase::from(seed_phrase);
        let key_pair = KeyPair::try_new(seed_phrase)?;

        let enc_sk = encrypt_data(&pass_hash.cipher, key_pair.private_key.as_bytes())?;
        let secret_key = hex::encode(enc_sk.as_slice());

        let public_key = key_pair.get_pk();

        Ok((
            Self::new(Some(public_key), Some(secret_key), Some(salt)),
            key_pair,
        ))
    }

    pub async fn load() -> anyhow::Result<Self> {
        load_from_local_storage(&STORAGE_KEYS).await
    }

    pub async fn save(self) -> anyhow::Result<()> {
        save_to_local_storage(self).await
    }

    pub async fn remove() -> anyhow::Result<()> {
        remove_from_local_storage(&STORAGE_KEYS).await
    }
}

pub enum StorageCredentialsAction {
    Add(CredentialsMap, Password),
    Edit(CredentialsMap, Password),
    Delete(CredentialsMap, Password),
    Logout,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageCredentials {
    pub credentials: CredentialsMap,
}

impl TryInto<Object> for StorageCredentials {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Object, Self::Error> {
        let js_value = JsValue::from_serde(&self)?;
        Ok(Object::from(js_value))
    }
}

impl From<Vec<Password>> for StorageCredentials {
    fn from(passwords: Vec<Password>) -> Self {
        let mut credentials = CredentialsMap::new();
        for password in passwords {
            let map = credentials.entry(password.site.clone()).or_default();
            map.insert(password._id.clone(), password);
        }
        Self { credentials }
    }
}

impl StorageCredentials {
    pub fn new(credentials: CredentialsMap) -> Self {
        Self { credentials }
    }

    pub async fn load() -> anyhow::Result<Self> {
        load_from_local_storage(&CREDENTIALS_KEYS).await
    }

    pub async fn save(self) -> anyhow::Result<()> {
        save_to_local_storage(self).await
    }

    pub async fn remove() -> anyhow::Result<()> {
        remove_from_local_storage(&CREDENTIALS_KEYS).await
    }
}

impl StorageCredentialsAction {
    pub async fn execute(self, api: &Api) -> anyhow::Result<()> {
        match self {
            StorageCredentialsAction::Add(credentials, password) => {
                api.add_password(password.user_id.clone(), password).await?;

                let credentials = StorageCredentials::new(credentials);
                credentials.save().await
            }
            StorageCredentialsAction::Edit(credentials, password) => {
                api.edit_password(password.user_id, password._id, password.password)
                    .await?;
                let credentials = StorageCredentials::new(credentials);
                credentials.save().await
            }
            StorageCredentialsAction::Delete(credentials, password) => {
                api.delete_password(password.user_id, password._id).await?;
                let credentials = StorageCredentials::new(credentials);
                credentials.save().await
            }
            StorageCredentialsAction::Logout => {
                StorageSecretKey::remove().await?;
                StorageCredentials::remove().await?;
                Ok(())
            }
        }
    }

    pub async fn execute_without_api(self) -> anyhow::Result<()> {
        match self {
            StorageCredentialsAction::Logout => {
                StorageSecretKey::remove().await?;
                StorageCredentials::remove().await?;
                Ok(())
            }
            _ => Err(anyhow!("Cannot execute action without API")),
        }
    }
}

async fn save_to_local_storage(
    obj: impl TryInto<Object, Error = anyhow::Error>,
) -> anyhow::Result<()> {
    let obj: Object = obj.try_into()?;
    chrome()
        .storage()
        .local()
        .set(&obj)
        .await
        .map_err(|err| anyhow!("Error writing to local storage: {:?}", err))?;

    Ok(())
}

async fn load_from_local_storage<T: for<'a> Deserialize<'a>>(keys: &[&str]) -> anyhow::Result<T> {
    let js_value = chrome()
        .storage()
        .local()
        .get(&JsValue::from_serde(keys)?)
        .await
        .map_err(|err| anyhow!("Error reading from local storage: {:?}", err))?;

    js_value
        .into_serde()
        .map_err(|err| anyhow!("Error deserializing local storage: {:?}", err))
}

async fn remove_from_local_storage(keys: &[&str]) -> anyhow::Result<()> {
    chrome()
        .storage()
        .local()
        .remove(&JsValue::from_serde(keys)?)
        .await
        .map_err(|err| anyhow!("Error reading from local storage: {:?}", err))?;

    Ok(())
}

pub async fn execute_storage_credentials_action(
    app: &Rc<RefCell<App>>,
    action: StorageCredentialsAction,
) -> anyhow::Result<()> {
    let api = match app.borrow().get_api() {
        Ok(api) => api,
        Err(err) => {
            return Err(anyhow!("Error getting API: {:?}", err));
        }
    };

    action.execute(&api).await
}
