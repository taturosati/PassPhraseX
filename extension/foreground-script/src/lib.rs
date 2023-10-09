use gloo_utils::document;
use gloo_utils::format::JsValueSerdeExt;

use gloo_console as console;
use messages::PortResponsePayload;
use wasm_bindgen::{prelude::*, JsCast};
use web_extensions_sys::{chrome, Port};
use web_sys::{window, Element};

#[wasm_bindgen]
pub fn start() {
    let port = connect();

    let (username_input, password_input) = match get_inputs() {
        Some(inputs) => inputs,
        None => return,
    };

    let on_message = move |msg: JsValue| {
        let msg: messages::Response<PortResponsePayload> =
            gloo_utils::format::JsValueSerdeExt::into_serde(&msg).unwrap();

        match &msg.payload {
            PortResponsePayload::Credential { username, password } => {
                username_input
                    .set_attribute("value", username)
                    .expect("Failed to set username");

                password_input
                    .set_attribute("value", password)
                    .expect("Failed to set password");
            }
            _ => {}
        }
    };

    let closure: Closure<dyn Fn(JsValue)> = Closure::new(on_message);
    let callback = closure.as_ref().unchecked_ref();
    port.on_message().add_listener(callback);
    closure.forget();

    if let Some(site) = get_site() {
        let payload = messages::PortRequestPayload::GetCredential { site };
        let msg = JsValue::from_serde(&messages::Request::new(payload)).unwrap();
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

fn get_inputs() -> Option<(Element, Element)> {
    match document()
        .query_selector("input[type=\"password\"]")
        .unwrap_or(None)
    {
        Some(password_input) => match password_input.closest("form").unwrap_or(None) {
            Some(form) => form
                .query_selector("input[type=\"text\"], input[name=\"email\"]")
                .unwrap_or(None)
                .map(|username_input| (username_input, password_input)),
            None => None,
        },
        None => None,
    }
}

fn connect() -> Port {
    let connect_info = JsValue::null();
    chrome()
        .runtime()
        .connect(None, connect_info.as_ref().unchecked_ref())
}
