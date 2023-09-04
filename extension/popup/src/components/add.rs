use messages::AppRequestPayload;
use web_sys::HtmlInputElement;
use yew::{function_component, html, use_node_ref, Callback, Html, Properties};

use crate::api::app_request;
use gloo_console as console;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cb: Callback<()>,
}

#[function_component]
pub fn Add(props: &Props) -> Html {
    let site_ref = use_node_ref();
    let username_ref = use_node_ref();
    let password_ref = use_node_ref();

    let onclick = {
        let site_ref = site_ref.clone();
        let username_ref = username_ref.clone();
        let password_ref = password_ref.clone();
        let cb = props.cb.clone();

        move |_| {
            let input = site_ref.cast::<HtmlInputElement>();
            let site = input.map(|input| input.value());

            let input = username_ref.cast::<HtmlInputElement>();
            let username = input.map(|input| input.value());

            let input = password_ref.cast::<HtmlInputElement>();
            let password = input.map(|input| input.value());

            if site.is_none() || username.is_none() || password.is_none() {
                return;
            }

            let site = site.unwrap();
            let username = username.unwrap();
            let password = password.unwrap();

            let payload = AppRequestPayload::AddCredential {
                site,
                username,
                password,
            };

            let cb = cb.clone();
            app_request(payload, move |res| match res {
                Ok(_) => {
                    cb.emit(());
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
                <label for="site">{"Site"}</label>
                <input type="text" id="Site" name="site" ref={site_ref} />
                <label for="username">{"Username"}</label>
                <input type="text" id="username" name="username" ref={username_ref} />
                <label for="password">{"Password"}</label>
                <input type="text" id="password" name="password" ref={password_ref} />
            </form>
            <button {onclick}>{ "Add" }</button>
        </div>
    }
}
