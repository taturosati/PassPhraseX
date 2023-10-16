use gloo_console as console;
use gloo_utils::document;
use messages::{PortRequestPayload, PortResponsePayload};
use wasm_bindgen::{prelude::*, JsCast};
use web_extensions_sys::{chrome, Port};
use web_sys::{window, Element, MutationObserver, MutationObserverInit};

#[wasm_bindgen]
pub fn start() {
    console::log!("Starting");
    let port = connect();

    let on_change = {
        let port = port.clone();
        move |_| {
            console::log!("Changed");

            let (username_input, password_input) = get_inputs();
            if username_input.is_none() && password_input.is_none() {
                return;
            }

            console::log!("Found inputs");

            let on_message = move |msg: JsValue| {
                let username_input = username_input.clone();
                let password_input = password_input.clone();

                let msg: messages::Response<PortResponsePayload> =
                    serde_wasm_bindgen::from_value(msg).unwrap();

                match &msg.payload {
                    PortResponsePayload::Credential { username, password } => {
                        if let Some(username_input) = username_input {
                            console::log!("Setting username", username.clone());
                            username_input
                                .set_attribute("value", username)
                                .expect("Failed to set username");
                        }

                        if let Some(password_input) = password_input {
                            console::log!("Setting password", password.clone());
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
    };

    let closure: Closure<dyn Fn(JsValue)> = Closure::new(on_change.clone());
    let callback = closure.as_ref().unchecked_ref();

    let mutation_observer = MutationObserver::new(callback).expect("TODO: panic message");

    if let Some(body) = document().body() {
        let mut options = MutationObserverInit::new();
        options.child_list(true);
        options.subtree(true);
        // options.character_data(true);
        // options.attributes(true);
        match mutation_observer.observe_with_options(body.as_ref(), &options) {
            Ok(_) => console::log!("Successfully started observer"),
            Err(e) => console::log!("Failed to start observer", e),
        }
    } else {
        console::log!("Failed to get body");
    }

    on_change(JsValue::null());
    closure.forget();
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
