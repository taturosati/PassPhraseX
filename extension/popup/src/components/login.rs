use yew::{function_component, html, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_login: Callback<()>,
}

#[function_component]
pub fn Login(props: &Props) -> Html {
    let onclick = {
        let on_login = props.on_login.clone();
        move |_| {
            on_login.emit(());
        }
    };

    html! {
        <div>
            <h1>{ "Login" }</h1>
            <input type="text" placeholder="seed phrase" />
            <button {onclick}>{ "Login" }</button>
        </div>
    }
}
