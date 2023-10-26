mod api;
mod components;
mod pages;

use gloo_console as console;
use wasm_bindgen::prelude::*;

use crate::api::app_request;
use crate::pages::{PageProps, Pages, Render};
use messages::{AppRequestPayload, AppResponsePayload};
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let payload = AppRequestPayload::GetStatus;
    let current_page = use_state_eq(|| Pages::Login);

    use_effect_with_deps(
        {
            let cb = {
                let current_page = current_page.clone();

                move |response| match response {
                    Ok(AppResponsePayload::Status {
                        is_logged_in,
                        is_unlocked,
                    }) => {
                        if is_unlocked {
                            current_page.set(Pages::Unlocked);
                        } else if is_logged_in {
                            current_page.set(Pages::Unlock);
                        } else {
                            current_page.set(Pages::Login);
                        }
                    }
                    Err(err) => {
                        console::error!("Error: {:?}", err);
                        current_page.set(Pages::Error);
                    }
                    _ => {
                        current_page.set(Pages::Error);
                    }
                }
            };

            move |_| {
                app_request(payload, cb);
            }
        },
        (),
    );

    let set_page = {
        let current_page = current_page.clone();

        Callback::from(move |page: Pages| {
            current_page.set(page);
        })
    };

    current_page.render(&PageProps { set_page })
}

#[wasm_bindgen]
pub fn start() {
    yew::Renderer::<App>::new().render();
}
