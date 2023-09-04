use crate::components::login::Login;
use crate::pages::{PageProps, Pages};
use yew::{function_component, html, Callback, Html};

#[function_component]
pub fn LoggedOutApp(props: &PageProps) -> Html {
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
