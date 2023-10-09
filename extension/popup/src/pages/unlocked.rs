use crate::components::add::Add;
use crate::components::edit::Edit;
use crate::components::list::List;
use crate::components::nav::Nav;
use crate::pages::{PageProps, Render};
use messages::Credential;
use yew::{function_component, html, use_state, Html, Properties, UseStateHandle};

#[derive(Properties, PartialEq)]
pub struct SectionProps {
    pub section: UseStateHandle<Sections>,
}

#[derive(PartialEq, Clone)]
pub enum Sections {
    Add,
    List,
    Edit(Credential),
}

impl Render<SectionProps> for Sections {
    fn render(&self, props: &SectionProps) -> Html {
        let section = props.section.clone();
        match self {
            Sections::Add => {
                html!(<Add {section}/>)
            }
            Sections::List => {
                html!(<List {section} />)
            }
            Sections::Edit(cred) => {
                html!(<Edit {section} credential={cred.clone()} />)
            }
        }
    }
}

#[function_component]
pub fn UnlockedApp(_props: &PageProps) -> Html {
    let section = use_state(|| Sections::List);

    let child = section.render(&SectionProps {
        section: section.clone(),
    });

    html! {
        <div>
            <Nav {section}/>
            {child}
        </div>
    }
}
