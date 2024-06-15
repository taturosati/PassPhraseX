use crate::app::App;
use anyhow::anyhow;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Object;
use passphrasex_common::api::Api;
use passphrasex_common::crypto::asymmetric::{KeyPair, SeedPhrase};
use passphrasex_common::model::password::Password;
use passphrasex_common::model::CredentialsMap;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_extensions_sys::chrome;

pub static STORAGE_KEYS: [&str; 2] = ["private_key", "signing_key"];
pub static CREDENTIALS_KEYS: [&str; 1] = ["credentials"];

#[derive(Clone)]
pub struct KeyPairOption(pub Option<KeyPair>);

#[derive(Serialize, Deserialize, Clone)]
pub struct StorageKeys {
    private_key: Vec<u8>,
    signing_key: Vec<u8>,
}

impl StorageKeys {
    pub(crate) fn try_into_key_pair(self, password: &str) -> anyhow::Result<KeyPair> {
        let key_pair = KeyPair::try_from_private_keys(
            self.private_key.as_slice(),
            self.signing_key.as_slice(),
            password,
        )?;

        Ok(key_pair)
    }

    pub(crate) async fn load() -> anyhow::Result<Self> {
        load_from_local_storage(&STORAGE_KEYS)
            .await
            .map_err(|err| anyhow!("Error loading storage keys: {:?}", err))
    }

    pub(crate) async fn save(self) -> anyhow::Result<()> {
        save_to_local_storage(self).await
    }
}

impl TryInto<Object> for StorageKeys {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Object, Self::Error> {
        let js_value = JsValue::from_serde(&self)?;
        Ok(Object::from(js_value))
    }
}

impl KeyPairOption {
    pub async fn generate() -> anyhow::Result<(Self, String)> {
        let seed_phrase = SeedPhrase::new();
        let key_pair = KeyPair::try_new(seed_phrase.clone())?;

        let verifying_key = key_pair.get_verifying_key();

        let api = Api::new(key_pair.clone());
        api.create_user(verifying_key).await?;

        Ok((Self(Some(key_pair)), seed_phrase.get_phrase()))
    }

    pub async fn from_seed_phrase(seed_phrase: String) -> anyhow::Result<Self> {
        let seed_phrase = SeedPhrase::from(seed_phrase);
        let key_pair = KeyPair::try_new(seed_phrase)?;

        Ok(Self(Some(key_pair)))
    }

    pub async fn has_key_pair() -> anyhow::Result<bool> {
        load_from_local_storage::<StorageKeys>(&STORAGE_KEYS)
            .await
            .map(|_| true)
            .or_else(|_| Ok(false))
    }

    pub async fn save(self, device_pass: &str) -> anyhow::Result<()> {
        match self {
            Self(Some(key_pair)) => {
                let private_key = key_pair.get_private_key_enc(device_pass);
                let signing_key = key_pair.get_signing_key_enc(device_pass);

                let storage_keys = StorageKeys {
                    private_key,
                    signing_key,
                };

                storage_keys.save().await
            }
            Self(None) => remove_from_local_storage(&STORAGE_KEYS).await,
        }
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
                KeyPairOption::remove().await?;
                StorageCredentials::remove().await?;
                Ok(())
            }
        }
    }

    pub async fn execute_without_api(self) -> anyhow::Result<()> {
        match self {
            StorageCredentialsAction::Logout => {
                KeyPairOption::remove().await?;
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
