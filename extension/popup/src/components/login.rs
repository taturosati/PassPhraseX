use crate::api::try_auth;
use crate::components::helpers::{button::Button, input::Input};
use messages::AppRequestPayload;
use yew::{function_component, html, use_state, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cb: Callback<()>,
}

#[function_component]
pub fn Login(props: &Props) -> Html {
    let seed_phrase = use_state(|| "".to_string());
    let device_password = use_state(|| "".to_string());
    let error = use_state(|| Some("".to_string()));

    let onclick = {
        let seed_phrase = (*seed_phrase).clone();
        let device_password = (*device_password).clone();
        let cb = props.cb.clone();
        let error = error.clone();

        move |_| {
            let error = error.clone();
            let cb = cb.clone();
            let seed_phrase = seed_phrase.clone();
            let device_password = device_password.clone();

            try_login(
                seed_phrase,
                device_password,
                move |payload: Option<String>| {
                    if payload.is_some() {
                        return error.set(Some("Invalid Credentials".to_string()));
                    }
                    cb.emit(());
                },
            )
        }
    };

    html! {
        <div>
            <form>
                <Input input_type="text" value={seed_phrase} label={"Seed Phrase"}/>
                <Input input_type="password" value={device_password} label={"Device Password"}/>
            </form>
            {(*error).clone().map(|error| html! { <p class={"text-red-500 text-xs mb-2"}>{error}</p> })}
            <Button {onclick} text={"Login"} />
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
