use crate::components::add::Add;
use crate::components::edit::Edit;
use crate::components::list::List;
use crate::components::nav::{Nav, NavTab, NavTabButtonProps};
use crate::pages::{PageProps, Pages, Render};
use messages::{AppRequestPayload, Credential};
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
pub fn UnlockedApp(props: &PageProps) -> Html {
    let set_page = props.set_page.clone();
    let section = use_state(|| Sections::List);

    let child = section.render(&SectionProps {
        section: section.clone(),
    });

    let tabs = [
        NavTab {
            text: "List".to_string(),
            section: Sections::List,
            button: None,
        },
        NavTab {
            text: "Add".to_string(),
            section: Sections::Add,
            button: None,
        },
        NavTab {
            text: "Lock".to_string(),
            section: Sections::List,
            button: Some(NavTabButtonProps {
                set_page: set_page.clone(),
                page: Pages::Unlock,
                payload: AppRequestPayload::Lock {},
            }),
        },
        NavTab {
            text: "Logout".to_string(),
            section: Sections::List,
            button: Some(NavTabButtonProps {
                set_page: set_page.clone(),
                page: Pages::Login,
                payload: AppRequestPayload::Logout {},
            }),
        },
    ]
    .to_vec();

    html! {
        <div>
            <Nav<Sections> {section} {set_page} {tabs} />
            {child}
        </div>
    }
}
