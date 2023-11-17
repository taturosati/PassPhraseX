use crate::api::app_request;
use crate::components::helpers::{button::Button, input::Input};
use messages::{AppRequestPayload, AppResponsePayload};
use yew::{function_component, html, use_state, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cb: Callback<String>,
}

#[function_component]
pub fn Register(props: &Props) -> Html {
    let device_password = use_state(|| "".to_string());
    let error = use_state(|| Some("".to_string()));

    let onclick = {
        let device_password = (*device_password).clone();
        let cb = props.cb.clone();
        let error = error.clone();

        move |_| {
            let error = error.clone();
            let cb = cb.clone();
            let device_password = device_password.clone();

            try_register(
                device_password,
                move |res: Result<String, String>| match res {
                    Ok(seed_phrase) => {
                        cb.emit(seed_phrase);
                    }
                    Err(err) => {
                        error.set(Some(err));
                    }
                },
            )
        }
    };

    html! {
        <div>
            <form>
                <Input input_type="password" value={device_password} label={"Device Password"}/>
            </form>
            {(*error).clone().map(|error| html! { <p class={"text-red-500 text-xs mb-2"}>{error}</p> })}
            <Button {onclick} text={"Register"} />
        </div>
    }
}

fn try_register<F>(device_password: String, callback: F)
where
    F: Fn(Result<String, String>) + 'static,
{
    let payload = AppRequestPayload::Register { device_password };
    app_request(payload, move |result| match result {
        Ok(payload) => match payload {
            AppResponsePayload::Auth { error: Some(error) } => {
                callback(Err(error));
            }
            AppResponsePayload::SeedPhrase(seed_phrase) => {
                callback(Ok(seed_phrase));
            }
            _ => {
                callback(Err("Invalid Response".to_string()));
            }
        },
        Err(err) => {
            callback(Err(err));
        }
    });
}
