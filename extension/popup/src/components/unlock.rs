use crate::api::try_auth;
use messages::AppRequestPayload;
use web_sys::HtmlInputElement;
use yew::{function_component, html, use_node_ref, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_unlock: Callback<()>,
}

#[function_component]
pub fn Unlock(props: &Props) -> Html {
    let device_password_ref = use_node_ref();

    let onclick = {
        let device_password_ref = device_password_ref.clone();
        let on_unlock = props.on_unlock.clone();

        move |_| {
            let input = device_password_ref.cast::<HtmlInputElement>();
            let device_password = input.map(|input| input.value());
            let on_unlock = on_unlock.clone();

            if let Some(device_password) = device_password {
                try_unlock(device_password, move |payload: Option<String>| {
                    if payload.is_some() {
                        // TODO: ERROR
                        return;
                    }

                    on_unlock.emit(());
                });
            }
        }
    };

    html! {
        <div>
            <h1>{ "Unlock" }</h1>
            <input type="password" placeholder="device password" ref={device_password_ref}/>
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
