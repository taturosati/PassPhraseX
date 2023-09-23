use messages::AppRequestPayload;
use yew::{function_component, html, use_state, Html};

use crate::api::app_request;
use crate::components::helpers::{button::Button, input::Input};
use crate::pages::unlocked::{SectionProps, Sections};
use gloo_console as console;

#[function_component]
pub fn Add(props: &SectionProps) -> Html {
    let site = use_state(|| "".to_string());
    let username = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());

    let onclick = {
        let site = site.clone();
        let username = username.clone();
        let password = password.clone();
        let section = props.section.clone();

        move |_| {
            let site = (*site).clone();
            let username = (*username).clone();
            let password = (*password).clone();

            let payload = AppRequestPayload::AddCredential {
                site,
                username,
                password,
            };

            let section = section.clone();
            app_request(payload, move |res| match res {
                Ok(_) => {
                    section.set(Sections::List);
                }
                Err(err) => {
                    console::error!("Error: {:?}", err);
                }
            });
        }
    };

    html! {
        <div>
            <form>
                <Input label="Site" value={site} />
                <Input label="Username" value={username} />
                <Input label="Password" value={password} />
                <Button {onclick} text={"Add Credential"} />
            </form>
        </div>
    }
}
