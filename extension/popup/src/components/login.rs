use crate::api::app_request;
use messages::{AppRequestPayload, AppResponsePayload};
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
                try_login(value, move |payload: Option<String>| {
                    if let Some(payload) = payload {
                        // error
                        text.set(payload);
                        return;
                    }

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
    F: Fn(Option<String>) + 'static,
{
    let payload = AppRequestPayload::Login(seed_phrase);

    app_request(payload, move |result| match result {
        Ok(payload) => {
            if let AppResponsePayload::Login { success, error } = payload {
                if success {
                    callback(None);
                } else if let Some(error) = error {
                    callback(Some(error));
                } else {
                    callback(Some("UNKNOWN_ERR".to_string()));
                }
            }
        }
        Err(err) => {
            callback(Some(err));
        }
    });

    Ok(())
}

// fn handle_response(response: JsValue) -> Result<AppResponsePayload, String> {
//     let response: Response<AppResponsePayload> =
//         response.into_serde().map_err(|err| err.to_string())?;
//
//     match response.payload {
//         AppResponsePayload::Login { success, error } => {
//             if success {
//                 Ok(AppResponsePayload::Login { success, error })
//             } else {
//                 Err("ERR_FAIL".to_string())
//             }
//         }
//         _ => Err("ERR_APP_RESPONSE".to_string()),
//     }
// }
