use crate::pages::unlocked::{SectionProps, Sections};
use material_yew::{MatTab, MatTabBar};
use yew::{function_component, html, Callback, Html};

#[function_component]
pub fn Nav(props: &SectionProps) -> Html {
    let set_section = &props.set_section;

    let onactivated = {
        let set_section = set_section.clone();
        Callback::from(move |index| match index {
            0 => set_section.emit(Sections::Add),
            1 => set_section.emit(Sections::List),
            2 => set_section.emit(Sections::Edit),
            _ => {}
        })
    };

    html! {
        <MatTabBar onactivated={onactivated}>
            <MatTab label={"ADD"}></MatTab>
            <MatTab label={"LIST"}></MatTab>
            <MatTab label={"Edit"}></MatTab>
        </MatTabBar>
    }
}
