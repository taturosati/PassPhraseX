use crate::storage::{StorageCredentials, StorageCredentialsAction, StorageSecretKey};
use crate::{ConnectedPorts, PortError, PortId};
use anyhow::anyhow;
use messages::{next_request_id, Credential, RequestId};
use passphrasex_common::api::Api;
use passphrasex_common::crypto::asymmetric::KeyPair;
use passphrasex_common::model::password::Password;
use passphrasex_common::model::CredentialsMap;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_extensions_sys::Port;

pub struct UnlockedAppData {
    key_pair: KeyPair,
    credentials_map: CredentialsMap,
    tmp_credentials: HashMap<String, (String, String)>,
    api: Api,
}

#[derive(Default)]
pub enum AppData {
    #[default]
    Locked,
    Unlocked(UnlockedAppData),
}

impl AppData {
    fn new(key_pair: KeyPair, credentials_map: CredentialsMap) -> Self {
        let api = Api::new(key_pair.clone());

        Self::Unlocked(UnlockedAppData {
            key_pair,
            credentials_map,
            tmp_credentials: HashMap::new(),
            api,
        })
    }
}

#[derive(Default)]
pub struct App {
    pub last_request_id: RequestId,
    pub connected_ports: ConnectedPorts,
    pub app_data: AppData,
}

impl App {
    pub fn next_request_id(&mut self) -> RequestId {
        let next_request_id = next_request_id(self.last_request_id);
        self.last_request_id = next_request_id;
        next_request_id
    }

    pub fn connect_port(&mut self, port: Port) -> Option<PortId> {
        self.connected_ports.connect(port)
    }

    pub fn disconnect_port(&mut self, port_id: PortId) -> Option<Port> {
        self.connected_ports.disconnect(port_id)
    }

    pub fn next_port_request_id(&mut self, port_id: PortId) -> Result<RequestId, PortError> {
        self.connected_ports.next_request_id(port_id)
    }

    pub fn post_port_message_js(&self, port_id: PortId, msg: &JsValue) -> Result<(), PortError> {
        self.connected_ports.post_message_js(port_id, msg)
    }

    pub fn get_status(&self, sk: StorageSecretKey) -> anyhow::Result<(bool, bool)> {
        match self.app_data {
            AppData::Locked => match sk.secret_key {
                Some(_) => Ok((true, false)),
                None => Ok((false, false)),
            },
            AppData::Unlocked { .. } => Ok((true, true)),
        }
    }

    pub fn get_api(&self) -> anyhow::Result<Api> {
        match &self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => Ok(app_data.api.clone()),
        }
    }

    pub fn unlock(
        &mut self,
        sk: StorageSecretKey,
        creds: StorageCredentials,
        device_password: String,
    ) -> anyhow::Result<()> {
        let public_key = sk.public_key.ok_or(anyhow!("No pk found"))?;
        let private_key = sk.secret_key.ok_or(anyhow!("No sk found"))?;


        let key_pair = KeyPair::try_from_private_keys(private_key.as_slice(), device_password.as_str())?;
        if key_pair.get_verifying_key() != public_key {
            return Err(anyhow!("Invalid key pair"));
        }

        let credentials_map = creds.credentials;

        match self.app_data {
            AppData::Locked => {
                self.app_data = AppData::new(key_pair, credentials_map);
            }
            AppData::Unlocked { .. } => {
                return Err(anyhow!("Already unlocked"));
            }
        }

        Ok(())
    }

    pub fn lock(&mut self) -> anyhow::Result<()> {
        match self.app_data {
            AppData::Locked => {
                return Err(anyhow!("Already locked"));
            }
            AppData::Unlocked { .. } => {
                self.app_data = AppData::Locked;
            }
        }

        Ok(())
    }

    pub fn login(&mut self, key_pair: KeyPair, credentials: CredentialsMap) {
        self.app_data = AppData::new(key_pair, credentials);
    }

    pub fn logout(&mut self) -> StorageCredentialsAction {
        self.app_data = AppData::Locked;
        StorageCredentialsAction::Logout
    }

    pub fn get_credential(
        &self,
        site: String,
        username: Option<String>,
    ) -> anyhow::Result<(String, String)> {
        match &self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => match app_data.credentials_map.get(&site) {
                Some(passwords) => match username {
                    Some(username) => {
                        let id = app_data.key_pair.hash(&format!("{}{}", site, username));
                        let credential = passwords.get(&id).ok_or(anyhow!("Password not found"))?;
                        let credential = credential.decrypt(&app_data.key_pair);
                        Ok((credential.username, credential.password))
                    }
                    None => {
                        let result: Vec<Password> = passwords
                            .iter()
                            .map(|(_, password)| password.decrypt(&app_data.key_pair))
                            .collect();

                        let result = result.first().ok_or(anyhow!("No passwords found"))?;
                        Ok((result.username.clone(), result.password.clone()))
                    }
                },
                None => Err(anyhow!("No passwords found")),
            },
        }
    }

    pub fn list_credentials(&self) -> anyhow::Result<Vec<Credential>> {
        match &self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => {
                let result: Vec<Credential> = app_data
                    .credentials_map
                    .iter()
                    .flat_map(|(_, creds)| {
                        creds
                            .iter()
                            .map(|(_, password)| password.decrypt(&app_data.key_pair))
                            .collect::<Vec<Password>>()
                    })
                    .map(|cred| Credential {
                        id: cred._id.clone(),
                        site: cred.site.clone(),
                        username: cred.username.clone(),
                        password: cred.password,
                    })
                    .collect();

                Ok(result)
            }
        }
    }

    pub fn add_credential(
        &mut self,
        site: String,
        username: String,
        password: String,
    ) -> anyhow::Result<StorageCredentialsAction> {
        match &mut self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => {
                if username.is_empty() || site.is_empty() {
                    return Err(anyhow!("Username & site cannot be empty"));
                }

                let password_id = app_data.key_pair.hash(&format!("{}{}", site, username));
                let user_id = app_data.key_pair.get_verifying_key();

                let password = Password {
                    _id: password_id.clone(),
                    user_id,
                    site: site.clone(),
                    username,
                    password,
                };

                let password = password.encrypt(&app_data.key_pair);
                app_data
                    .credentials_map
                    .entry(site)
                    .or_insert(HashMap::new())
                    .insert(password_id, password.clone());

                let action =
                    StorageCredentialsAction::Add(app_data.credentials_map.clone(), password);

                Ok(action)
            }
        }
    }

    pub fn edit_credential(
        &mut self,
        site: String,
        password_id: String,
        new_password: String,
    ) -> anyhow::Result<StorageCredentialsAction> {
        match &mut self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => {
                let mut password = app_data
                    .credentials_map
                    .get_mut(&site)
                    .ok_or(anyhow!("No site found"))?
                    .get_mut(&password_id)
                    .ok_or(anyhow!("No password found"))?
                    .decrypt(&app_data.key_pair);

                password.password = new_password;
                let password = password.encrypt(&app_data.key_pair);
                app_data
                    .credentials_map
                    .entry(site)
                    .or_insert(HashMap::new())
                    .insert(password_id, password.clone());

                let action =
                    StorageCredentialsAction::Edit(app_data.credentials_map.clone(), password);
                Ok(action)
            }
        }
    }

    pub fn add_or_edit_credential(
        &mut self,
        site: String,
        username: String,
        password: String,
    ) -> anyhow::Result<StorageCredentialsAction> {
        match &self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => {
                let site_map = app_data.credentials_map.get(&site);

                if password.is_empty() {
                    return Err(anyhow!("Password cannot be empty"));
                }

                match site_map {
                    Some(site_map) => match site_map.values().next() {
                        Some(pass) => {
                            let password_id = pass._id.clone();
                            self.edit_credential(site, password_id, password)
                        }
                        None => self.add_credential(site, username, password),
                    },
                    None => self.add_credential(site, username, password),
                }

                //
                //
                //
                // let has_password = app_data
                //     .credentials_map
                //     .get(&site)
                //     .map_or(false, |map| map.contains_key(&password_id));
                //
                // if has_password {
                //     self.edit_credential(site, password_id, password)
                // } else {
                //     self.add_credential(site, username, password)
                // }
            }
        }
    }

    pub fn delete_credential(
        &mut self,
        site: String,
        password_id: String,
    ) -> anyhow::Result<StorageCredentialsAction> {
        match &mut self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => {
                let password = app_data
                    .credentials_map
                    .get_mut(&site)
                    .ok_or(anyhow!("No site found"))?
                    .remove(&password_id)
                    .ok_or(anyhow!("No password found"))?;

                let action =
                    StorageCredentialsAction::Delete(app_data.credentials_map.clone(), password);
                Ok(action)
            }
        }
    }

    pub fn set_tmp_credential_username(
        &mut self,
        site: String,
        username: String,
    ) -> anyhow::Result<()> {
        match &mut self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => {
                let mut password = "".to_string();
                if let Some((_, p)) = app_data.tmp_credentials.get(&site) {
                    password.clone_from(p);
                }
                app_data.tmp_credentials.insert(site, (username, password));

                Ok(())
            }
        }
    }

    pub fn set_tmp_credential_password(
        &mut self,
        site: String,
        password: String,
    ) -> anyhow::Result<()> {
        match &mut self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => {
                let mut username = "".to_string();
                if let Some((u, _)) = app_data.tmp_credentials.get(&site) {
                    username.clone_from(u);
                }
                app_data.tmp_credentials.insert(site, (username, password));

                Ok(())
            }
        }
    }

    pub fn store_tmp_credential(
        &mut self,
        site: String,
    ) -> anyhow::Result<StorageCredentialsAction> {
        match &mut self.app_data {
            AppData::Locked => Err(anyhow!("Not Logged In")),
            AppData::Unlocked(app_data) => {
                let (username, password) = app_data
                    .tmp_credentials
                    .remove(&site)
                    .ok_or(anyhow!("No tmp credential found"))?;

                self.add_or_edit_credential(site, username, password)
            }
        }
    }
}
