use crate::pages::unlocked::{SectionProps, Sections};
use yew::{
    classes, function_component, html, use_effect_with_deps, use_state, Callback, Html, Properties,
    UseStateHandle,
};

#[function_component]
pub fn Nav(props: &SectionProps) -> Html {
    let section = props.section.clone();

    html! {
        <ul class={classes!("mb-2", "flex", "list-none", "flex-row", "flex-wrap", "border-b-0", "pl-0")} role="tablist">
            <Tab section={section.clone()} this_section={Sections::List} text="List" />
            <Tab section={section.clone()} this_section={Sections::Add} text="Add" />
            <Tab section={section.clone()} this_section={Sections::Edit} text="Edit" />
        </ul>
    }
}

#[derive(Properties, PartialEq)]
struct TabProps {
    pub section: UseStateHandle<Sections>,
    pub this_section: Sections,
    pub text: String,
}

#[function_component]
fn Tab(props: &TabProps) -> Html {
    let section = props.section.clone();
    let classes = use_state(|| "border-transparent text-neutral-500");
    let this_section = props.this_section.clone();

    let onclick = {
        let section = section.clone();
        let this_section = this_section.clone();

        Callback::from(move |_| {
            section.set(this_section.clone());
        })
    };

    use_effect_with_deps(
        {
            let section = section.clone();
            let classes = classes.clone();
            let this_section = this_section.clone();

            move |_| {
                if *section == this_section {
                    classes.set("border-blue-700 text-blue-700");
                } else {
                    classes.set("border-transparent text-neutral-500");
                }
            }
        },
        section,
    );

    html! {
        <li role="presentation" class="flex-grow basis-0 text-center">
            <a {onclick} role="tab"
              class={"mb-2 block border-x-0 border-b-2 border-t-0 px-7 pb-3.5 pt-4 text-xs font-medium uppercase leading-tight hover:isolate hover:bg-neutral-100 cursor-pointer focus:isolate focus:border-transparent ".to_string() + *classes }
            >{props.text.clone()}</a>
        </li>
    }
}
