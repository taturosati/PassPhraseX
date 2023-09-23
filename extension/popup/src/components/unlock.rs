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
    let device_password_state = use_state(String::new);
    let device_password = (*device_password_state).clone();

    let onclick = {
        let device_password = device_password.clone();
        let on_unlock = props.on_unlock.clone();

        move |_| {
            let device_password = device_password.clone();
            let on_unlock = on_unlock.clone();

            try_unlock(device_password, move |payload: Option<String>| {
                if payload.is_some() {
                    // TODO: ERROR
                    return;
                }

                on_unlock.emit(());
            });
        }
    };

    html! {
        <div>
            <Input label="Device Password" value={device_password_state} />
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
