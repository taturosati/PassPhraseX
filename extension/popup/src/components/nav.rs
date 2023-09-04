use crate::pages::unlocked::{SectionProps, Sections};
use yew::{function_component, html, Callback, Html};

#[function_component]
pub fn Nav(props: &SectionProps) -> Html {
    let set_section = props.set_section.clone();

    let set_section_add = {
        let set_section = set_section.clone();
        Callback::from(move |_| set_section.emit(Sections::Add))
    };

    let set_section_list = {
        let set_section = set_section.clone();
        Callback::from(move |_| set_section.emit(Sections::List))
    };

    let set_section_edit = {
        let set_section = set_section.clone();
        Callback::from(move |_| set_section.emit(Sections::Edit))
    };

    html! {
        <nav>
            <button onclick={set_section_add}>{"Add"}</button>
            <button onclick={set_section_list}>{"List"}</button>
            <button onclick={set_section_edit}>{"Edit"}</button>
        </nav>
    }
}
