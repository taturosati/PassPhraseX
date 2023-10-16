use crate::api::try_auth;
use crate::components::helpers::{button::Button, input::Input};
use messages::AppRequestPayload;
use yew::{function_component, html, use_state, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_unlock: Callback<()>,
}

#[function_component]
pub fn Unlock(props: &Props) -> Html {
    let error_state = use_state(|| Option::<String>::None);
    let error = (*error_state).clone();

    let device_password_state = use_state(String::new);
    let device_password = (*device_password_state).clone();

    let onclick = {
        let device_password = device_password.clone();
        let on_unlock = props.on_unlock.clone();

        move |_| {
            let error_state = error_state.clone();
            let device_password = device_password.clone();
            let on_unlock = on_unlock.clone();

            try_unlock(device_password, move |payload: Option<String>| {
                if payload.is_some() {
                    error_state.set(Some("Invalid device password".to_string()));
                    return;
                }

                on_unlock.emit(());
            });
        }
    };

    html! {
        <div>
            <Input input_type="password" label="Device Password" value={device_password_state} error={error} />
            <Button {onclick} text="Unlock"></Button>
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
