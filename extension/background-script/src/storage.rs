use anyhow::anyhow;
// use gloo_console as console;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Object;
use passphrasex_common::api::Api;
use passphrasex_common::model::password::Password;
use passphrasex_common::model::CredentialsMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_extensions_sys::chrome;

pub static STORAGE_KEYS: [&str; 2] = ["secret_key", "salt"];
pub static CREDENTIALS_KEYS: [&str; 1] = ["credentials"];

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageSecretKey {
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
    pub fn new(secret_key: Option<String>, salt: Option<String>) -> Self {
        Self { secret_key, salt }
    }

    pub async fn load() -> anyhow::Result<Self> {
        // let js_value = chrome()
        //     .storage()
        //     .local()
        //     .get(&JsValue::from_serde(&STORAGE_KEYS)?)
        //     .await
        //     .map_err(|err| anyhow!("Error reading from local storage: {:?}", err))?;
        //
        // js_value
        //     .into_serde()
        //     .map_err(|err| anyhow!("Error deserializing local storage: {:?}", err))
        load_from_local_storage(&STORAGE_KEYS).await
    }

    pub async fn save(self) -> anyhow::Result<()> {
        save_to_local_storage(self).await
    }
}

pub enum StorageCredentialsAction {
    Add(CredentialsMap, Password),
    Edit(CredentialsMap, Password),
    Delete(CredentialsMap, Password),
}

#[derive(Debug, Serialize, Deserialize)]
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

impl StorageCredentials {
    fn new(credentials: CredentialsMap) -> Self {
        Self { credentials }
    }

    pub async fn load() -> anyhow::Result<Self> {
        load_from_local_storage(&CREDENTIALS_KEYS).await
    }

    async fn save(self) -> anyhow::Result<()> {
        save_to_local_storage(self).await
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
