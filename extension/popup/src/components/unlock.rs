use crate::api::{app_request, try_auth};
use crate::components::helpers::button::ButtonVariants;
use crate::components::helpers::{button::Button, input::Input};
use messages::AppRequestPayload;
use yew::{function_component, html, use_state, Callback, Html, Properties};

pub enum Msg {
    Unlock,
    Logout,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cb: Callback<Msg>,
}

#[function_component]
pub fn Unlock(props: &Props) -> Html {
    let error_state = use_state(|| Option::<String>::None);
    let error = (*error_state).clone();

    let device_password_state = use_state(String::new);
    let device_password = (*device_password_state).clone();

    let onclick = {
        let device_password = device_password.clone();
        let cb = props.cb.clone();

        move |_| {
            let error_state = error_state.clone();
            let device_password = device_password.clone();
            let cb = cb.clone();

            try_unlock(device_password, move |payload: Option<String>| {
                if payload.is_some() {
                    error_state.set(Some("Invalid device password".to_string()));
                    return;
                }

                cb.emit(Msg::Unlock);
            });
        }
    };

    let logout = {
        let cb = props.cb.clone();

        move |_| {
            let cb = cb.clone();
            let payload = AppRequestPayload::Logout {};

            app_request(payload, move |res| {
                if res.is_err() {
                    gloo_console::log!("Error logging out", res.err());
                    return;
                }
                cb.emit(Msg::Logout);
            });
        }
    };

    html! {
        <div>
            <Input input_type="password" label="Device Password" value={device_password_state} error={error} />
            <Button {onclick} text="Unlock" class="mb-2" />
            <Button onclick={logout} text="Logout" variant={ButtonVariants::Dark} />
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
