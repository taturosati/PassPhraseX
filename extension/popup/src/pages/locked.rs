use crate::components::unlock::{Msg, Unlock};
use crate::pages::{PageProps, Pages};
use yew::{function_component, html, Callback, Html};

#[function_component]
pub fn LockedApp(props: &PageProps) -> Html {
    let cb = Callback::from({
        let set_page = props.set_page.clone();
        move |msg| match msg {
            Msg::Unlock => set_page.emit(Pages::Unlocked),
            Msg::Logout => set_page.emit(Pages::Login),
        }
    });

    html! {
        <Unlock {cb} />
    }
}
