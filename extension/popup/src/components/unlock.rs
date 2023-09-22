use crate::api::try_auth;
use crate::components::input::Input;
use messages::AppRequestPayload;
use wasm_bindgen::UnwrapThrowExt;
use yew::{function_component, html, use_state, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_unlock: Callback<()>,
}

#[function_component]
pub fn Unlock(props: &Props) -> Html {
    let device_password_state = use_state(String::new);
    let device_password = (*device_password_state).clone();

    let onclick = {
        let device_password = device_password.clone();
        let on_unlock = props.on_unlock.clone();

        move |_| {
            let device_password = device_password.clone();
            // let input = div_ref.cast::<HtmlInputElement>();
            // let device_password = input.map(|input| input.value());
            let on_unlock = on_unlock.clone();

            // if let Some(device_password) = device_password {
            try_unlock(device_password, move |payload: Option<String>| {
                if payload.is_some() {
                    // TODO: ERROR
                    return;
                }

                on_unlock.emit(());
            });
            // }
        }
    };

    html! {
        <div>
            <h1>{ "Unlock" }</h1>
            <Input label="Device Password" value={device_password_state} />
            <button {onclick}>{ "Unlock" }</button>
        </div>
    }
}

fn try_unlock<F>(device_password: String, callback: F)
where
    F: Fn(Option<String>) + 'static,
{
    let payload = AppRequestPayload::Unlock { device_password };
    try_auth(payload, callback);
}
