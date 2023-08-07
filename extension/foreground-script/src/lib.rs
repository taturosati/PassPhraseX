use gloo_console as console;
use gloo_utils::format::JsValueSerdeExt;
use gloo_utils::{body, document};
use wasm_bindgen::{prelude::*, JsCast};
use web_extensions_sys::{chrome, Port};
use web_sys::console::log_1;
use web_sys::Element;

#[wasm_bindgen]
pub fn start() {
    // render_container();
    let port = connect();

    let on_message = |msg: JsValue| {
        console::info!("Received message:", msg);
    };
    let closure: Closure<dyn Fn(JsValue)> = Closure::new(on_message);
    let callback = closure.as_ref().unchecked_ref();
    port.on_message().add_listener(callback);
    closure.forget();

    let (username_input, password_input) = match get_inputs() {
        Some(inputs) => inputs,
        None => return,
    };

    username_input
        .set_attribute("value", "test@test.com")
        .expect("Failed to set username");

    password_input
        .set_attribute("value", "test")
        .expect("Failed to set password");

    // let closure: Closure<dyn Fn(JsValue)> = Closure::new(on_change);
    // let callback = closure.as_ref().unchecked_ref();
    //
    // password_input
    //     .add_event_listener_with_callback("change", callback)
    //     .expect("TODO: panic message");

    // closure.forget();

    // let payload = messages::PortRequestPayload::Ping;
    // let msg = JsValue::from_serde(&messages::Request::new(payload)).unwrap();
    // port.post_message(&msg);
    //
    // let payload = messages::PortRequestPayload::StartStreaming { num_items: 5 };
    // let msg = JsValue::from_serde(&messages::Request::new(payload)).unwrap();
    // port.post_message(&msg);
}

fn get_inputs() -> Option<(Element, Element)> {
    match document()
        .query_selector("input[type=\"password\"]")
        .unwrap_or(None)
    {
        Some(password_input) => match password_input.closest("form").unwrap_or(None) {
            Some(form) => match form
                .query_selector("input[type=\"text\"], input[name=\"email\"]")
                .unwrap_or(None)
            {
                Some(username_input) => {
                    log_1(&username_input);
                    Some((username_input, password_input))
                }
                None => None,
            },
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
