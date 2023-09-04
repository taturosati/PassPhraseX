use messages::{AppRequestPayload, AppResponsePayload};
use yew::{function_component, html, use_effect_with_deps, use_state, Html};

use crate::api::app_request;
use crate::pages::unlocked::SectionProps;
use gloo_console as console;

#[function_component]
pub fn List(_: &SectionProps) -> Html {
    let credentials = use_state(Vec::new);

    use_effect_with_deps(
        {
            let credentials = credentials.clone();

            move |_| {
                let payload = AppRequestPayload::ListCredentials {};
                app_request(payload, move |res| match res {
                    Ok(response) => match response {
                        AppResponsePayload::Credentials(creds) => {
                            credentials.set(creds);
                        }
                        _ => {
                            console::error!("Error: {:?}", "Invalid response");
                        }
                    },
                    Err(err) => {
                        console::error!("Error: {:?}", err);
                    }
                });
            }
        },
        (),
    );
    html! {
        <div>
            {credentials.iter().map(|credential| {
                html! {
                    <div>
                        <div>{credential.site.clone()}</div>
                        <div>{credential.username.clone()}</div>
                        <div>{credential.password.clone()}</div>
                    </div>
                }
            }).collect::<Html>()}
        </div>
    }
}
