use crate::components::unlock::Unlock;
use crate::pages::{PageProps, Pages};
use yew::{function_component, html, Callback, Html};

#[function_component]
pub fn LockedApp(props: &PageProps) -> Html {
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
