use gloo_utils::document;

use gloo_console as console;
use messages::{PortRequestPayload, PortResponsePayload};
use wasm_bindgen::{prelude::*, JsCast};
use web_extensions_sys::{chrome, Port};
use web_sys::{window, Element};

#[wasm_bindgen]
pub fn start() {
    let port = connect();

    let on_message = move |msg: JsValue| {
        let (username_input, password_input) = get_inputs();

        let msg: messages::Response<PortResponsePayload> =
            serde_wasm_bindgen::from_value(msg).unwrap();

        match &msg.payload {
            PortResponsePayload::Credential { username, password } => {
                if let Some(username_input) = username_input {
                    username_input
                        .set_attribute("value", username)
                        .expect("Failed to set username");
                }

                if let Some(password_input) = password_input {
                    password_input
                        .set_attribute("value", password)
                        .expect("Failed to set password");
                }
            }
            _ => {}
        }
    };

    let closure: Closure<dyn Fn(JsValue)> = Closure::new(on_message);
    let callback = closure.as_ref().unchecked_ref();

    port.on_message().add_listener(callback);
    closure.forget();

    if let Some(site) = get_site() {
        console::log!("Site: {}", site.clone());
        let payload = PortRequestPayload::GetCredential { site };
        let msg = serde_wasm_bindgen::to_value(&messages::Request::new(payload)).unwrap();
        port.post_message(&msg);
    }
}

fn get_site() -> Option<String> {
    match window().as_ref() {
        Some(window) => match window.location().href() {
            Ok(href) => {
                let href = href
                    .replace("https://", "")
                    .replace("http://", "")
                    .replace("www.", "");

                match href.split("/").next() {
                    Some(site) => Some(site.to_string()),
                    None => None,
                }
            }
            Err(_) => None,
        },
        None => None,
    }
}

fn get_inputs() -> (Option<Element>, Option<Element>) {
    (
        get_input("input[name=\"email\"], input[name=\"username\"]"),
        get_input("input[type=\"password\"], input[name=\"password\"]"),
    )
}

fn get_input(selector: &str) -> Option<Element> {
    match document().query_selector(selector).unwrap_or(None) {
        Some(username_input) => Some(username_input),
        None => None,
    }
}

fn connect() -> Port {
    let connect_info = JsValue::null();
    chrome()
        .runtime()
        .connect(None, connect_info.as_ref().unchecked_ref())
}
