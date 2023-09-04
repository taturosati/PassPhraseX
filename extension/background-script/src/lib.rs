mod storage;

use anyhow::anyhow;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gloo_console as console;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::{Function, Object};
use messages::{
    next_request_id, AppRequest, AppRequestPayload, AppResponse, AppResponsePayload, Credential,
    PortRequest, PortRequestPayload, PortResponse, PortResponsePayload, Request, RequestId,
    Response, INITIAL_REQUEST_ID,
};
use serde::Serialize;
use thiserror::Error;
use wasm_bindgen::{prelude::*, JsCast};

use crate::storage::StorageSecretKey;
use passphrasex_common::crypto::asymmetric::{KeyPair, SeedPhrase};
use passphrasex_common::crypto::symmetric::{decrypt_data, encrypt_data, generate_salt, hash};
use passphrasex_common::model::password::Password;
use passphrasex_common::model::CredentialsMap;
use web_extensions_sys::{chrome, Port, Tab, TabChangeInfo};

const VERSION: &str = env!("CARGO_PKG_VERSION");

type TabId = i32;

type PortId = usize;

const FIRST_PORT_ID: RequestId = 1;

#[derive(Debug)]
struct PortContext {
    port: Port,
    last_request_id: RequestId,
}

impl PortContext {
    const fn new(port: Port) -> Self {
        Self {
            port,
            last_request_id: INITIAL_REQUEST_ID,
        }
    }

    fn next_request_id(&mut self) -> RequestId {
        let next_request_id = next_request_id(self.last_request_id);
        self.last_request_id = next_request_id;
        next_request_id
    }
}

#[derive(Default)]
struct ConnectedPorts {
    last_id: PortId,
    ctx_by_id: HashMap<PortId, PortContext>,
}

#[derive(Debug, Error)]
enum PortError {
    #[error("not connected")]
    NotConnected,
}

impl ConnectedPorts {
    fn connect(&mut self, port: Port) -> Option<PortId> {
        let id = self.last_id.checked_add(1)?;
        debug_assert!(id >= FIRST_PORT_ID);
        let ctx = PortContext::new(port);
        self.ctx_by_id.insert(id, ctx);
        Some(id)
    }

    fn disconnect(&mut self, id: PortId) -> Option<Port> {
        self.ctx_by_id
            .remove(&id)
            .map(|PortContext { port, .. }| port)
    }

    fn post_message_js(&self, id: PortId, msg: &JsValue) -> Result<(), PortError> {
        self.ctx_by_id
            .get(&id)
            .ok_or(PortError::NotConnected)
            .map(|ctx| {
                let PortContext {
                    port,
                    last_request_id: _,
                } = ctx;
                console::debug!("Posting message on port", port, msg);
                port.post_message(msg);
            })
    }

    fn next_request_id(&mut self, id: PortId) -> Result<RequestId, PortError> {
        self.ctx_by_id
            .get_mut(&id)
            .ok_or(PortError::NotConnected)
            .map(|ctx| ctx.next_request_id())
    }
}

struct AppData {
    key_pair: KeyPair,
    credentials_map: CredentialsMap,
}

#[derive(Default)]
struct App {
    last_request_id: RequestId,
    connected_ports: ConnectedPorts,
    app_data: Option<AppData>,
}

impl App {
    fn next_request_id(&mut self) -> RequestId {
        let next_request_id = next_request_id(self.last_request_id);
        self.last_request_id = next_request_id;
        next_request_id
    }

    fn connect_port(&mut self, port: Port) -> Option<PortId> {
        self.connected_ports.connect(port)
    }

    fn disconnect_port(&mut self, port_id: PortId) -> Option<Port> {
        self.connected_ports.disconnect(port_id)
    }

    fn next_port_request_id(&mut self, port_id: PortId) -> Result<RequestId, PortError> {
        self.connected_ports.next_request_id(port_id)
    }

    fn post_port_message_js(&self, port_id: PortId, msg: &JsValue) -> Result<(), PortError> {
        self.connected_ports.post_message_js(port_id, msg)
    }

    fn get_status(&self, sk: StorageSecretKey) -> anyhow::Result<(bool, bool)> {
        if self.app_data.is_some() {
            // Logged in & unlocked
            return Ok((true, true));
        }

        if sk.secret_key.is_none() {
            // Not logged in
            return Ok((false, false));
        }

        Ok((true, false))
    }

    fn unlock(&mut self, sk: StorageSecretKey, device_password: String) -> anyhow::Result<()> {
        let salt = sk.salt.ok_or(anyhow!("No salt found"))?;
        let sk = sk.secret_key.ok_or(anyhow!("No sk found"))?;
        let sk = hex::decode(sk).map_err(|err| anyhow!("Unable to decode sk: {:?}", err))?;

        let pass_hash = hash(&device_password, &salt)?;

        let sk = decrypt_data(&pass_hash.cipher, sk)?;

        let mut content: [u8; 32] = [0; 32];
        content.copy_from_slice(&sk[..32]);

        let key_pair = KeyPair::from_sk(content);
        match self.app_data.as_mut() {
            Some(app_data) => {
                app_data.key_pair = key_pair;
            }
            None => {
                self.app_data = Some(AppData {
                    key_pair,
                    credentials_map: CredentialsMap::new(),
                });
            }
        }

        Ok(())
    }

    fn login(
        &mut self,
        seed_phrase: String,
        device_password: String,
    ) -> anyhow::Result<StorageSecretKey> {
        let salt = generate_salt()?;
        let pass_hash = hash(&device_password, &salt)?;

        let seed_phrase = SeedPhrase::from(seed_phrase);
        let key_pair = KeyPair::new(seed_phrase);

        let enc_sk = encrypt_data(&pass_hash.cipher, key_pair.private_key.as_bytes())?;
        let encoded_sk = hex::encode(enc_sk.as_slice());

        let key_storage = StorageSecretKey::new(Some(encoded_sk), Some(salt));

        self.app_data = Some(AppData {
            key_pair,
            credentials_map: CredentialsMap::new(),
        });

        Ok(key_storage)
    }

    fn get_credential(
        &self,
        site: String,
        username: Option<String>,
    ) -> anyhow::Result<(String, String)> {
        let app_data = self.app_data.as_ref().ok_or(anyhow!("Not Logged In"))?;
        let credentials = &app_data.credentials_map;
        match credentials.get(&site) {
            Some(passwords) => match username {
                Some(username) => {
                    let id = app_data.key_pair.hash(&format!("{}{}", site, username))?;
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
        }
    }

    fn list_credentials(&self) -> anyhow::Result<Vec<Credential>> {
        let app_data = self.app_data.as_ref().ok_or(anyhow!("Not Logged In"))?;
        let credentials = &app_data.credentials_map;
        let result: Vec<Credential> = credentials
            .iter()
            .flat_map(|(_, creds)| {
                creds
                    .iter()
                    .map(|(_, password)| password.decrypt(&app_data.key_pair))
                    .collect::<Vec<Password>>()
            })
            .map(|cred| Credential {
                site: cred.site.clone(),
                username: cred.username.clone(),
                password: cred.password,
            })
            .collect();

        Ok(result)
    }

    fn add_credential(
        &mut self,
        site: String,
        username: String,
        password: String,
    ) -> anyhow::Result<()> {
        let app_data = self.app_data.as_mut().ok_or(anyhow!("Not Logged In"))?;
        let password_id = app_data.key_pair.hash(&format!("{}{}", site, username))?;
        let user_id = app_data.key_pair.get_pk();
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
            .insert(password_id, password);

        // TODO: Save to API & Local Storage
        Ok(())
    }
}

#[wasm_bindgen]
pub fn start() {
    console::info!("Starting background script");

    let app = Rc::new(RefCell::new(App::default()));

    let on_message = {
        let app = Rc::clone(&app);
        move |request, sender, send_response| on_message(&app, request, sender, send_response)
    };
    let closure: Closure<dyn Fn(JsValue, JsValue, Function) -> bool> = Closure::new(on_message);
    chrome()
        .runtime()
        .on_message()
        .add_listener(closure.as_ref().unchecked_ref());
    closure.forget();

    let closure: Closure<dyn Fn(TabId, TabChangeInfo, Tab)> = Closure::new(on_tab_changed);
    chrome()
        .tabs()
        .on_updated()
        .add_listener(closure.as_ref().unchecked_ref());
    closure.forget();

    let on_connect = move |port| {
        on_connect_port(&app, port);
    };
    let closure: Closure<dyn Fn(Port)> = Closure::new(on_connect);
    chrome()
        .runtime()
        .on_connect()
        .add_listener(closure.as_ref().unchecked_ref());
    closure.forget();
}

fn on_message(
    app: &Rc<RefCell<App>>,
    request: JsValue,
    sender: JsValue,
    send_response: Function,
) -> bool {
    console::debug!("Received request message", &request, &sender);
    let request_id = app.borrow_mut().next_request_id();

    {
        let app = app.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let future = on_request(&app, request_id, request);

            if let Some(response) = future.await {
                console::debug!("Sending response message", &response, &sender);
                let this = JsValue::null();
                if let Err(err) = send_response.call1(&this, &response) {
                    console::error!(
                        "Failed to send response message",
                        send_response,
                        response,
                        err
                    );
                }
            }
        });

        true
    }
}

fn on_tab_changed(tab_id: i32, change_info: TabChangeInfo, tab: Tab) {
    console::info!("Tab changed", tab_id, &tab, &change_info);
    if change_info.status().as_deref() == Some("complete") {
        if let Some(url) = tab.url() {
            if url.starts_with("http") {
                console::info!("Injecting foreground script on tab", tab_id, &tab);
                wasm_bindgen_futures::spawn_local(inject_frontend(tab_id));
            }
        }
    }
}

fn on_connect_port(app: &Rc<RefCell<App>>, port: Port) {
    console::info!("Connecting new port", &port);
    let port_id = if let Some(port_id) = app.borrow_mut().connect_port(port.clone()) {
        port_id
    } else {
        console::error!("Failed to connect new port", &port);
        return;
    };
    let on_message = {
        let app = Rc::clone(app);
        move |request| {
            on_port_message(&app, port_id, request);
        }
    };
    let closure: Closure<dyn Fn(JsValue)> = Closure::new(on_message);
    port.on_message()
        .add_listener(closure.as_ref().unchecked_ref());
    closure.forget();

    let on_disconnect = {
        let app = Rc::clone(app);
        move || {
            console::log!(format!("Port {port_id} has disconnected"));
            app.borrow_mut().disconnect_port(port_id);
        }
    };
    let closure: Closure<dyn Fn()> = Closure::new(on_disconnect);
    port.on_disconnect()
        .add_listener(closure.as_ref().unchecked_ref());
    closure.forget();
}

fn on_port_message(app: &Rc<RefCell<App>>, port_id: PortId, request: JsValue) {
    console::debug!("Received request message on port", port_id, &request);
    let request_id = match app.borrow_mut().next_port_request_id(port_id) {
        Ok(request_id) => request_id,
        Err(err) => {
            console::warn!(
                "Failed to handle port request",
                port_id,
                request,
                err.to_string()
            );
            return;
        }
    };
    if let Some(response) = on_port_request(app, port_id, request_id, request) {
        if let Err(err) = app.borrow().post_port_message_js(port_id, &response) {
            console::warn!(
                "Failed to post response message to port",
                port_id,
                response,
                err.to_string()
            );
        }
    }
}

async fn on_request(
    app: &Rc<RefCell<App>>,
    request_id: RequestId,
    request: JsValue,
) -> Option<JsValue> {
    let request = request
        .into_serde()
        .map_err(|err| {
            console::error!("Failed to deserialize request message", &err.to_string());
        })
        .ok()?;
    let response = handle_app_request(app, request_id, request).await;
    JsValue::from_serde(&response)
        .map_err(|err| {
            console::error!("Failed to serialize response message", &err.to_string());
        })
        .ok()
}

fn on_port_request(
    app: &Rc<RefCell<App>>,
    port_id: PortId,
    request_id: RequestId,
    request: JsValue,
) -> Option<JsValue> {
    let request = request
        .into_serde()
        .map_err(|err| {
            console::error!(
                "Failed to deserialize port request message",
                &err.to_string()
            );
        })
        .ok()?;
    let response = handle_port_request(app, port_id, request_id, request);
    JsValue::from_serde(&response)
        .map_err(|err| {
            console::error!(
                "Failed to serialize port response message",
                &err.to_string()
            );
        })
        .ok()
}

/// Handle a (global) request.
///
/// Optionally returns a single response.
///
async fn handle_app_request(
    app: &Rc<RefCell<App>>,
    request_id: RequestId,
    request: AppRequest,
) -> Option<AppResponse> {
    let Request { header, payload } = request;
    let payload: AppResponsePayload = match payload {
        AppRequestPayload::GetOptionsInfo => AppResponsePayload::OptionsInfo {
            version: VERSION.to_string(),
        },
        AppRequestPayload::GetStatus => match StorageSecretKey::load().await {
            Ok(sk) => match app.borrow().get_status(sk) {
                Ok((is_logged_in, is_unlocked)) => AppResponsePayload::Status {
                    is_logged_in,
                    is_unlocked,
                },
                Err(_) => {
                    return None;
                }
            },
            Err(_) => {
                return None;
            }
        },
        AppRequestPayload::Unlock { device_password } => match StorageSecretKey::load().await {
            Ok(sk) => match app.borrow_mut().unlock(sk, device_password) {
                Ok(()) => AppResponsePayload::Auth { error: None },
                Err(err) => AppResponsePayload::Auth {
                    error: Some(err.to_string()),
                },
            },
            Err(err) => AppResponsePayload::Auth {
                error: Some(err.to_string()),
            },
        },
        AppRequestPayload::Login {
            seed_phrase,
            device_password,
        } => {
            let result = { app.borrow_mut().login(seed_phrase, device_password) };
            match result {
                Ok(key_storage) => match key_storage.save().await {
                    Ok(()) => AppResponsePayload::Auth { error: None },
                    Err(err) => AppResponsePayload::Auth {
                        error: Some(err.to_string()),
                    },
                },
                Err(err) => AppResponsePayload::Auth {
                    error: Some(err.to_string()),
                },
            }
        }
        AppRequestPayload::GetCredential { site, username } => {
            match app.borrow().get_credential(site, username) {
                Ok((username, password)) => AppResponsePayload::Credential { username, password },
                Err(err) => {
                    console::error!("Failed to get credential", err.to_string());
                    return None; // TODO: Error
                }
            }
        }
        AppRequestPayload::AddCredential {
            site,
            username,
            password,
        } => {
            match app
                .borrow_mut()
                .add_credential(site, username.clone(), password.clone())
            {
                Ok(()) => AppResponsePayload::Credential { username, password },
                Err(err) => {
                    console::error!("Failed to add credential", err.to_string());
                    return None; // TODO: Error
                }
            }
        }
        AppRequestPayload::ListCredentials {} => {
            match app.borrow().list_credentials() {
                Ok(credentials) => AppResponsePayload::Credentials(credentials),
                Err(err) => {
                    console::error!("Failed to list credentials", err.to_string());
                    return None; // TODO: Error
                }
            }
        }
    };

    Response {
        header: header.into_response(request_id),
        payload,
    }
    .into()
}

#[derive(Debug, Error)]
enum StreamingTaskError {
    #[error(transparent)]
    Port(#[from] PortError),
}

/// Handle a port-local request.
///
/// Optionally returns a single response.
///
/// TODO: Extract into domain crate
fn handle_port_request(
    app: &Rc<RefCell<App>>,
    _port_id: PortId,
    request_id: RequestId,
    request: PortRequest,
) -> Option<PortResponse> {
    let Request { header, payload } = request;
    let payload: Option<_> = match payload {
        PortRequestPayload::GetCredential { site } => {
            let (username, password) = app.borrow().get_credential(site, None).ok()?;
            PortResponsePayload::Credential { username, password }.into()
        }
    };
    // The started response might be posted after the first stream item response
    // or even after the finished response that are all generated asynchronously!
    payload.map(|payload| Response {
        header: header.into_response(request_id),
        payload,
    })
}

// https://developer.chrome.com/docs/extensions/reference/scripting/#type-CSSInjection
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CssInjection<'a> {
    target: InjectionTarget<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    css: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<&'a [&'a str]>,
}

// https://developer.chrome.com/docs/extensions/reference/scripting/#type-ScriptInjection
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScriptInjection<'a> {
    target: InjectionTarget<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<&'a [&'a str]>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InjectionTarget<'a> {
    tab_id: TabId,
    #[serde(skip_serializing_if = "Option::is_none")]
    all_frames: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frame_ids: Option<&'a [i32]>,
}

async fn inject_frontend(tab_id: TabId) {
    let css_injection = JsValue::from_serde(&CssInjection {
        files: Some(&["foreground-script/style.css"]),
        css: None,
        target: InjectionTarget {
            tab_id,
            all_frames: None,
            frame_ids: None,
        },
    })
    .unwrap();
    console::info!("Inject CSS", &css_injection);
    if let Err(err) = chrome()
        .scripting()
        .insert_css(&Object::from(css_injection))
        .await
    {
        console::info!("Unable to inject CSS", err);
    }
    let script_injection = JsValue::from_serde(&ScriptInjection {
        files: Some(&[
            "foreground-script/pkg/foreground_script.js",
            "foreground-script/index.js",
        ]),
        target: InjectionTarget {
            tab_id,
            all_frames: None,
            frame_ids: None,
        },
    })
    .unwrap();

    if let Err(err) = chrome()
        .scripting()
        .execute_script(&Object::from(script_injection))
        .await
    {
        console::info!("Unable to inject JS", err);
    }
}
