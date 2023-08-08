use gloo_console as console;
use gloo_utils::format::JsValueSerdeExt;

use messages::{AppRequestPayload, AppResponsePayload, Request, Response};
use wasm_bindgen::JsValue;
use web_extensions_sys::chrome;
use web_sys::HtmlInputElement;
use yew::{function_component, html, use_node_ref, use_state, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_login: Callback<()>,
}

#[function_component]
pub fn Login(props: &Props) -> Html {
    let input_ref = use_node_ref();
    let text = use_state(|| "".to_string());

    let onclick = {
        let input_ref = input_ref.clone();
        let on_login = props.on_login.clone();
        let text = text.clone();

        move |_| {
            let input = input_ref.cast::<HtmlInputElement>();
            let value = input.map(|input| input.value());

            let text = text.clone();
            let on_login = on_login.clone();

            if let Some(value) = value {
                try_login(value, move |payload: String| {
                    text.set(payload);
                    on_login.emit(());
                })
                .unwrap();
            }
        }
    };

    html! {
        <div>
            <h1>{ "Login" }</h1>
            <input type="text" placeholder="seed phrase" ref={input_ref} />
            <p>{ &*text }</p>
            <button {onclick}>{ "Login" }</button>
        </div>
    }
}

fn try_login<F>(seed_phrase: String, callback: F) -> Result<(), ()>
where
    F: Fn(String) + 'static,
{
    let payload = AppRequestPayload::Login(seed_phrase);
    let msg = JsValue::from_serde(&Request::new(payload)).map_err(|_| ())?;

    wasm_bindgen_futures::spawn_local(async move {
        match chrome().runtime().send_message(None, &msg, None).await {
            Ok(js_value) => {
                if js_value.is_object() {
                    let response = handle_response(js_value);
                    match response {
                        Ok(payload) => {
                            if let AppResponsePayload::Login { success, error } = payload {
                                if success {
                                    callback("".to_string());
                                } else {
                                    callback(error.unwrap());
                                }
                            }
                        }
                        Err(_) => {}
                    }
                } else {
                    console::debug!("The sender has unexpectedly not sent a reply");
                }
            }
            Err(err) => {
                console::error!("Unable to send request", err);
            }
        };
    });

    Ok(())
}

fn handle_response(response: JsValue) -> Result<AppResponsePayload, ()> {
    if let Ok(Response { header: _, payload }) = response.into_serde() {
        match payload {
            AppResponsePayload::Login { .. } => Ok(payload),
            _ => {
                console::warn!("Received unexpected message");
                Err(())
            }
        }
    } else {
        Err(())
    }
}
