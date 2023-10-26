use messages::{AppRequestPayload, AppResponsePayload};
use yew::{function_component, html, use_state, Html};

use crate::api::app_request;
use crate::components::helpers::{button::Button, input::Input};
use crate::pages::unlocked::{SectionProps, Sections};

#[function_component]
pub fn Add(props: &SectionProps) -> Html {
    let site = use_state(|| "".to_string());
    let username = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let error = use_state(|| None);

    let onclick = {
        let site = site.clone();
        let username = username.clone();
        let password = password.clone();
        let error = error.clone();
        let section = props.section.clone();

        move |_| {
            let site = (*site).clone();
            let username = (*username).clone();
            let password = (*password).clone();
            let error = error.clone();

            let payload = AppRequestPayload::AddCredential {
                site,
                username,
                password,
            };

            let section = section.clone();
            app_request(payload, move |res| match res {
                Ok(res) => match res {
                    AppResponsePayload::Credential { .. } => {
                        section.set(Sections::List);
                    }
                    AppResponsePayload::Error { .. } => {
                        error.set(Some("Credential already exists".to_string()));
                    }
                    _ => {
                        error.set(Some("Unknown Error".to_string()));
                    }
                },
                Err(err) => {
                    error.set(Some(err));
                }
            });
        }
    };

    html! {
        <div>
            <form>
                <Input label="Site" value={site} />
                <Input label="Username" value={username} />
                <Input input_type="password" label="Password" value={password} />
                {(*error).clone().map(|error| html! { <p class={"text-red-500 text-xs mb-2"}>{error}</p> })}
                <Button {onclick} text={"Add Credential"} />
            </form>
        </div>
    }
}
