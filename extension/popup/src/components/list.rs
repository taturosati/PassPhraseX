use messages::{AppRequestPayload, AppResponsePayload};
use yew::{function_component, html, use_effect_with_deps, use_state, Callback, Html, Properties};

use crate::api::app_request;
use crate::pages::unlocked::{SectionProps, Sections};
use gloo_console as console;

#[function_component]
pub fn List(props: &SectionProps) -> Html {
    let credentials = use_state(Vec::new);
    let section = props.section.clone();

    let counter = use_state(|| 0);

    let set_section = Callback::from({
        let counter = counter.clone();

        move |new_section| {
            if new_section == Sections::List {
                counter.set(*counter + 1);
            } else {
                section.set(new_section);
            }
        }
    });

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
        counter,
    );

    html! {
        <table class="table-auto w-full">
            <tbody>
                {credentials.iter().enumerate().map(|(idx, credential)| {
                    html! {
                        <Credential
                            credential={credential.clone()}
                            last={idx == credentials.len() - 1}
                            set_section={set_section.clone()}
                        />
                    }
                }).collect::<Html>()}
            </tbody>
        </table>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct CredentialProps {
    credential: messages::Credential,
    last: bool,
    set_section: Callback<Sections>,
}

#[function_component]
fn Credential(props: &CredentialProps) -> Html {
    let class = if props.last {
        "border-solid border-gray-200"
    } else {
        "border-solid border-gray-200 border-b-2"
    };

    let onclick_edit = Callback::from({
        let credential = props.credential.clone();

        let set_section = props.set_section.clone();

        move |_| {
            set_section.emit(Sections::Edit(credential.clone()));
        }
    });

    let onclick_delete = Callback::from({
        let set_section = props.set_section.clone();
        let credential = props.credential.clone();
        move |_| {
            let set_section = set_section.clone();

            let payload = AppRequestPayload::DeleteCredential {
                site: credential.site.clone(),
                password_id: credential.id.clone(),
            };

            app_request(payload, move |res| match res {
                Ok(_) => {
                    set_section.emit(Sections::List);
                }
                Err(err) => {
                    console::error!("Error: {:?}", err); // TODO: Error Page
                }
            });
        }
    });

    html! {
        <tr class={class}>
            <td>
                <div class="ms-2">
                    <div class="text-lg">{props.credential.site.clone()}</div>
                    <div>{props.credential.username.clone()}</div>
                </div>
            </td>
            <td class="text-center">
                <button class={"text-blue-700"} onclick={onclick_edit}>{"EDIT"}</button>
            </td>
            <td class="text-center">
                <button class={"text-red-500"} onclick={onclick_delete}>{"DELETE"}</button>
            </td>
        </tr>
    }
}
