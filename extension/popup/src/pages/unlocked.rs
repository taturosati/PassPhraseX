use crate::components::add::Add;
use crate::components::list::List;
use crate::components::nav::Nav;
use crate::pages::{PageProps, Render};
use yew::{function_component, html, use_state, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct SectionProps {
    pub set_section: Callback<Sections>,
}

pub enum Sections {
    Add,
    List,
    Edit,
}

impl Render<SectionProps> for Sections {
    fn render(&self, props: &SectionProps) -> Html {
        let set_section = props.set_section.clone();
        match self {
            Sections::Add => {
                html!(<Add {set_section}/>)
            }
            Sections::List => {
                html!(<List {set_section} />)
            }
            Sections::Edit => {
                html!(<div>{"Edit"}</div>)
            }
        }
    }
}

#[function_component]
pub fn UnlockedApp(_props: &PageProps) -> Html {
    let section = use_state(|| Sections::List);

    let set_section = {
        let section = section.clone();
        Callback::from(move |new_section| {
            section.set(new_section);
        })
    };

    let child = section.render(&SectionProps {
        set_section: set_section.clone(),
    });

    html! {
        <div>
            <Nav {set_section}/>
            {child}
        </div>
    }
}
