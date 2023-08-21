use anyhow::anyhow;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Object;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_extensions_sys::chrome;

pub static STORAGE_KEYS: [&str; 2] = ["secret_key", "salt"];

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
        let js_value = chrome()
            .storage()
            .local()
            .get(&JsValue::from_serde(&STORAGE_KEYS)?)
            .await
            .map_err(|err| anyhow!("Error reading from local storage: {:?}", err))?;

        js_value
            .into_serde()
            .map_err(|err| anyhow!("Error deserializing local storage: {:?}", err))
    }

    pub async fn save(self) -> anyhow::Result<()> {
        let obj: Object = self.try_into()?;
        chrome()
            .storage()
            .local()
            .set(&obj)
            .await
            .map_err(|err| anyhow!("Error writing to local storage: {:?}", err))?;

        Ok(())
    }
}
