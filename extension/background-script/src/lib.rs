mod app;
mod storage;

use app::App;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gloo_console as console;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Function;
use messages::{
    next_request_id, AppRequest, AppRequestPayload, AppResponse, AppResponsePayload, PortRequest,
    PortRequestPayload, PortResponse, PortResponsePayload, Request, RequestId, Response,
    INITIAL_REQUEST_ID,
};
use thiserror::Error;
use wasm_bindgen::{prelude::*, JsCast};

use crate::storage::{execute_storage_credentials_action, StorageCredentials, StorageSecretKey};
use passphrasex_common::api::Api;
use passphrasex_common::crypto::asymmetric::KeyPair;
use web_extensions_sys::{chrome, Port, Tab, TabChangeInfo};

const VERSION: &str = env!("CARGO_PKG_VERSION");

type TabId = i32;

type PortId = usize;

const FIRST_PORT_ID: RequestId = 1;

#[derive(Debug)]
pub struct PortContext {
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
pub struct ConnectedPorts {
    last_id: PortId,
    ctx_by_id: HashMap<PortId, PortContext>,
}

#[derive(Debug, Error)]
pub enum PortError {
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

        true // Need to return true to be able to use async stuff
    }
}

fn on_tab_changed(tab_id: i32, change_info: TabChangeInfo, tab: Tab) {
    console::info!("Tab changed", tab_id, &tab, &change_info);
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
        move |request| on_port_message(&app, port_id, request)
    };

    let closure: Closure<dyn Fn(JsValue) -> bool> = Closure::new(on_message);
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

fn on_port_message(app: &Rc<RefCell<App>>, port_id: PortId, request: JsValue) -> bool {
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
            return true;
        }
    };

    {
        let app = app.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let future = on_port_request(&app, port_id, request_id, request);

            if let Some(response) = future.await {
                if let Err(err) = app.borrow().post_port_message_js(port_id, &response) {
                    console::warn!(
                        "Failed to post response message to port",
                        port_id,
                        response,
                        err.to_string()
                    );
                }
            }
        });
    }

    true // Need to return true to be able to use async stuff
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

async fn on_port_request(
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
    let response = handle_port_request(app, port_id, request_id, request).await;
    JsValue::from_serde(&response)
        .map_err(|err| {
            console::error!(
                "Failed to serialize port response message",
                &err.to_string()
            );
        })
        .ok()
}

async fn auth(
    app: &Rc<RefCell<App>>,
    storage_key: StorageSecretKey,
    key_pair: KeyPair,
) -> AppResponsePayload {
    match Api::new(key_pair.clone())
        .get_passwords(key_pair.get_verifying_key())
        .await
    {
        Ok(passwords) => {
            let creds = StorageCredentials::from(passwords);
            app.borrow_mut().login(key_pair, creds.credentials.clone());

            match creds.save().await.and(storage_key.save().await) {
                Ok(()) => AppResponsePayload::Auth { error: None },
                Err(err) => AppResponsePayload::Auth {
                    error: Some(err.to_string()),
                },
            }
        }
        Err(err) => AppResponsePayload::Auth {
            error: Some(err.to_string()),
        },
    }
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
            Ok(sk) => match StorageCredentials::load().await {
                Ok(creds) => match app.borrow_mut().unlock(sk, creds, device_password) {
                    Ok(()) => AppResponsePayload::Auth { error: None },
                    Err(err) => AppResponsePayload::Auth {
                        error: Some(err.to_string()),
                    },
                },
                Err(err) => AppResponsePayload::Auth {
                    error: Some(err.to_string()),
                },
            },
            Err(err) => AppResponsePayload::Auth {
                error: Some(err.to_string()),
            },
        },
        AppRequestPayload::Lock {} => {
            match app.borrow_mut().lock() {
                Ok(()) => AppResponsePayload::Ok,
                Err(err) => {
                    console::error!("Failed to lock", err.to_string());
                    return None; // TODO: Error
                }
            }
        }
        AppRequestPayload::Login {
            seed_phrase,
            device_password,
        } => match StorageSecretKey::from_seed_phrase(seed_phrase, device_password).await {
            Ok((sk, key_pair)) => auth(app, sk, key_pair).await,
            Err(err) => AppResponsePayload::Auth {
                error: Some(err.to_string()),
            },
        },
        AppRequestPayload::Register { device_password } => {
            match StorageSecretKey::generate(device_password).await {
                Ok((sk, seed_phrase, key_pair)) => match auth(app, sk, key_pair).await {
                    AppResponsePayload::Auth { error: None } => {
                        AppResponsePayload::SeedPhrase(seed_phrase)
                    }
                    AppResponsePayload::Auth { error: Some(err) } => {
                        AppResponsePayload::Auth { error: Some(err) }
                    }
                    _ => {
                        return None;
                    }
                },
                Err(err) => AppResponsePayload::Auth {
                    error: Some(err.to_string()),
                },
            }
        }
        AppRequestPayload::Logout {} => {
            let logout_action = app.borrow_mut().logout();

            match logout_action.execute_without_api().await {
                Ok(()) => AppResponsePayload::Ok,
                Err(err) => AppResponsePayload::Error {
                    message: err.to_string(),
                },
            }
        }
        AppRequestPayload::GetCredential { site, username } => {
            match app.borrow().get_credential(site, username) {
                Ok((username, password)) => AppResponsePayload::Credential { username, password },
                Err(err) => AppResponsePayload::Error {
                    message: err.to_string(),
                },
            }
        }
        AppRequestPayload::AddCredential {
            site,
            username,
            password,
        } => {
            let result = {
                app.borrow_mut()
                    .add_credential(site, username.clone(), password.clone())
            };

            match result {
                Ok(action) => match execute_storage_credentials_action(app, action).await {
                    Ok(()) => AppResponsePayload::Credential { username, password },
                    Err(err) => AppResponsePayload::Error {
                        message: err.to_string(),
                    },
                },
                Err(err) => AppResponsePayload::Error {
                    message: err.to_string(),
                },
            }
        }
        AppRequestPayload::ListCredentials {} => match app.borrow().list_credentials() {
            Ok(credentials) => AppResponsePayload::Credentials(credentials),
            Err(err) => AppResponsePayload::Error {
                message: err.to_string(),
            },
        },
        AppRequestPayload::EditCredential {
            site,
            password_id,
            password,
        } => {
            let result = {
                app.borrow_mut()
                    .edit_credential(site, password_id.clone(), password.clone())
            };

            match result {
                Ok(action) => match execute_storage_credentials_action(app, action).await {
                    Ok(()) => AppResponsePayload::Ok,
                    Err(err) => AppResponsePayload::Error {
                        message: err.to_string(),
                    },
                },
                Err(err) => AppResponsePayload::Error {
                    message: err.to_string(),
                },
            }
        }
        AppRequestPayload::DeleteCredential { site, password_id } => {
            let result = { app.borrow_mut().delete_credential(site, password_id) };

            match result {
                Ok(action) => match execute_storage_credentials_action(app, action).await {
                    Ok(()) => AppResponsePayload::Ok,
                    Err(err) => AppResponsePayload::Error {
                        message: err.to_string(),
                    },
                },
                Err(err) => AppResponsePayload::Error {
                    message: err.to_string(),
                },
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
async fn handle_port_request(
    app: &Rc<RefCell<App>>,
    _port_id: PortId,
    request_id: RequestId,
    request: PortRequest,
) -> PortResponse {
    let Request { header, payload } = request;
    let payload = match payload {
        PortRequestPayload::GetCredential { site } => {
            match app.borrow().get_credential(site, None) {
                Ok((username, password)) => PortResponsePayload::Credential { username, password },
                Err(err) => {
                    console::error!("Failed to get credential", err.to_string());
                    PortResponsePayload::Error(err.to_string())
                }
            }
        }
        PortRequestPayload::SetTmpCredentialUsername { site, username } => {
            match app.borrow_mut().set_tmp_credential_username(site, username) {
                Ok(()) => PortResponsePayload::Ok,
                Err(err) => {
                    console::error!("Failed to set tmp credential username", err.to_string());
                    PortResponsePayload::Error(err.to_string())
                }
            }
        }
        PortRequestPayload::SetTmpCredentialPassword { site, password } => {
            match app.borrow_mut().set_tmp_credential_password(site, password) {
                Ok(()) => PortResponsePayload::Ok,
                Err(err) => {
                    console::error!("Failed to set tmp credential password", err.to_string());
                    PortResponsePayload::Error(err.to_string())
                }
            }
        }
        PortRequestPayload::StoreTmpCredential { site } => {
            let result = { app.borrow_mut().store_tmp_credential(site) };

            match result {
                Ok(action) => {
                    let result = execute_storage_credentials_action(app, action).await;
                    match result {
                        Ok(()) => PortResponsePayload::Ok,
                        Err(err) => {
                            console::error!("Failed to store tmp credential", err.to_string());
                            PortResponsePayload::Error(err.to_string())
                        }
                    }
                }
                Err(err) => {
                    console::error!("Failed to store tmp credential", err.to_string());
                    PortResponsePayload::Error(err.to_string())
                }
            }
        }
    };
    // The started response might be posted after the first stream item response
    // or even after the finished response that are all generated asynchronously!
    Response {
        header: header.into_response(request_id),
        payload,
    }
}
