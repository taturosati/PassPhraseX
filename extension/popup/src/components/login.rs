use crate::api::try_auth;
use messages::AppRequestPayload;
use web_sys::HtmlInputElement;
use yew::{function_component, html, use_node_ref, use_state, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_login: Callback<()>,
}

#[function_component]
pub fn Login(props: &Props) -> Html {
    let seed_phrase_ref = use_node_ref();
    let device_password_ref = use_node_ref();
    let text = use_state(|| "".to_string());

    let onclick = {
        let seed_phrase_ref = seed_phrase_ref.clone();
        let device_password_ref = device_password_ref.clone();
        let on_login = props.on_login.clone();
        let text = text.clone();

        move |_| {
            let input = seed_phrase_ref.cast::<HtmlInputElement>();
            let seed_phrase = input.map(|input| input.value());

            let input = device_password_ref.cast::<HtmlInputElement>();
            let device_password = input.map(|input| input.value());

            let text = text.clone();
            let on_login = on_login.clone();

            if let Some(seed_phrase) = seed_phrase {
                if let Some(device_password) = device_password {
                    try_login(
                        seed_phrase,
                        device_password,
                        move |payload: Option<String>| {
                            if let Some(payload) = payload {
                                // error
                                text.set(payload);
                                return;
                            }

                            on_login.emit(());
                        },
                    )
                }
            }
        }
    };

    html! {
        <div>
            <h1>{ "Login" }</h1>
            <input type="text" placeholder="seed phrase" ref={seed_phrase_ref} />
            <input type="password" placeholder="device password" ref={device_password_ref}/>
            <p>{ &*text }</p>
            <button {onclick}>{ "Login" }</button>
        </div>
    }
}

fn try_login<F>(seed_phrase: String, device_password: String, callback: F)
where
    F: Fn(Option<String>) + 'static,
{
    let payload = AppRequestPayload::Login {
        seed_phrase,
        device_password,
    };

    try_auth(payload, callback);
}
