mod api;
mod components;

use gloo_console as console;
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::*;
use web_extensions_sys::chrome;

use crate::components::login::Login;

use messages::{AppRequestPayload, AppResponsePayload, Request, Response};
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let auth = use_state(|| false);

    let on_login = Callback::from({
        let auth = auth.clone();
        move |_| {
            auth.set(true);
        }
    });

    html! {
        <div>
            <h1>{ "PassPhraseX" }</h1>
            <p>{ *auth }</p>
            <Login {on_login}/>
        </div>
    }
}

#[wasm_bindgen]
pub fn start() {
    console::info!("Start popup script");

    yew::Renderer::<App>::new().render();

    let payload = AppRequestPayload::GetOptionsInfo;
    let msg = JsValue::from_serde(&Request::new(payload)).unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        match chrome().runtime().send_message(None, &msg, None).await {
            Ok(js_value) => {
                if js_value.is_object() {
                    handle_response(js_value);
                } else {
                    console::debug!("The sender has unexpectedly not sent a reply");
                }
            }
            Err(err) => {
                console::error!("Unable to send request", err);
            }
        };
    });
}

fn handle_response(response: JsValue) {
    if let Ok(Response {
        header: _,
        payload: AppResponsePayload::OptionsInfo { version },
    }) = response.into_serde()
    {
        console::info!("Received version: {}", version);
    } else {
        console::warn!("Received unexpected message");
    }
}
