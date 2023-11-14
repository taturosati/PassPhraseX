use crate::api::app_request;
use messages::AppRequestPayload;
use yew::{function_component, html, use_state, Html, Properties, UseStateHandle};

use crate::components::helpers::{button::Button, input::Input};
use crate::pages::unlocked::Sections;

use gloo_console as console;

#[derive(PartialEq, Properties)]
pub struct EditProps {
    pub section: UseStateHandle<Sections>,
    pub credential: messages::Credential,
}

#[function_component]
pub fn Edit(props: &EditProps) -> Html {
    let id = props.credential.id.clone();
    let password = use_state(|| "".to_string());
    let site = props.credential.site.clone();

    let onclick = {
        let id = id.clone();
        let password = password.clone();
        let section = props.section.clone();

        move |_| {
            let password_id = id.clone();
            let password = (*password).clone();
            let section = section.clone();

            let payload = AppRequestPayload::EditCredential {
                password_id,
                password,
                site: site.clone(),
            };

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
    html!(
        <div>
            <h1 class={"text-lg"}>{"Edit"}</h1>
            <form>
                <Input label="New Password" input_type={"password"} value={password} />
                <Button {onclick} text={"Save"} />
            </form>
        </div>
    )
}
