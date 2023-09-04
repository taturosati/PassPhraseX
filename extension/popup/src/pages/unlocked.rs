use crate::components::add::Add;
use crate::pages::{PageProps, Pages};
use messages::{AppRequestPayload, AppResponsePayload};
use yew::{function_component, html, use_effect_with_deps, use_state, Callback, Html};

#[function_component]
pub fn UnlockedApp(props: &PageProps) -> Html {
    let on_logout = Callback::from({
        let set_page = props.set_page.clone();
        move |_| set_page.emit(Pages::Login)
    });

    let cred = use_state(|| ("".to_string(), "".to_string()));

    use_effect_with_deps(
        {
            let cred = cred.clone();

            move |_| {
                let payload = AppRequestPayload::GetCredential {
                    site: "test.com".to_string(),
                    username: None,
                };
                crate::api::app_request(payload, move |response| match response {
                    Ok(AppResponsePayload::Credential { username, password }) => {
                        cred.set((username, password));
                    }
                    Ok(_) => {
                        cred.set(("error".to_string(), "unknown response".to_string()));
                    }
                    Err(err) => {
                        cred.set(("error".to_string(), err));
                    }
                });
            }
        },
        (),
    );

    let cb = {
        let cred = cred.clone();
        move |_| {
            cred.set(("success".to_string(), "".to_string()));
        }
    };

    html! {
        <div>
            <h1>{ "PassPhraseX" }</h1>
            <div>{format!("Username: {}, Password: {}", cred.0, cred.1)}</div>
            <Add {cb}/>
            <button onclick={on_logout}>{"Logout"}</button>
        </div>
    }
}
