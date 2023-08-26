mod api;
mod components;

use gloo_console as console;
use wasm_bindgen::prelude::*;

use crate::components::login::Login;
use crate::components::unlock::Unlock;

use crate::api::app_request;
use messages::{AppRequestPayload, AppResponsePayload};
use yew::prelude::*;

#[derive(PartialEq)]
pub enum Pages {
    Login,
    Unlock,
    Unlocked,
}

#[derive(Properties, PartialEq)]
struct Props {
    set_page: Callback<Pages>,
}

#[function_component]
fn LoggedOutApp(props: &Props) -> Html {
    let on_login = Callback::from({
        let set_page = props.set_page.clone();
        move |_| set_page.emit(Pages::Unlocked)
    });

    html! {
        <div>
            <h1>{ "PassPhraseX" }</h1>
            <Login {on_login}/>
        </div>
    }
}

#[function_component]
fn LockedApp(props: &Props) -> Html {
    let on_unlock = Callback::from({
        let set_page = props.set_page.clone();
        move |_| {
            set_page.emit(Pages::Unlocked);
        }
    });

    html! {
        <div>
            <h1>{ "PassPhraseX" }</h1>
            <Unlock {on_unlock} />
        </div>
    }
}

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
                        // TODO: Error page
                        console::error!("Error: {:?}", err);
                    }
                    _ => {}
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

    match *current_page {
        Pages::Login => {
            html!(<LoggedOutApp {set_page}/>)
        }
        Pages::Unlock => {
            html!(<LockedApp {set_page}/>)
        }
        Pages::Unlocked => {
            html!(<div>{ "Unlocked" }</div>)
        }
    }
}

#[wasm_bindgen]
pub fn start() {
    yew::Renderer::<App>::new().render();
}
