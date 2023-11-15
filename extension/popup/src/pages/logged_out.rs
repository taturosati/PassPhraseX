use crate::components::login::Login;
use crate::components::nav::{Nav, NavTab};
use crate::components::register::Register;
use crate::components::seed_phrase::SeedPhrase;
use crate::pages::{PageProps, Pages};
use yew::{function_component, html, use_state, Callback, Html};

#[derive(PartialEq, Clone)]
pub enum Sections {
    Login {
        cb: Callback<()>,
    },
    Register {
        cb: Callback<String>,
    },
    SeedPhrase {
        seed_phrase: String,
        cb: Callback<()>,
    },
}

impl Sections {
    pub fn render(&self) -> Html {
        match self {
            Sections::Login { cb } => html!(<Login {cb}/>),
            Sections::Register { cb } => html!(<Register {cb}/>),
            Sections::SeedPhrase { cb, seed_phrase } => {
                html!(<SeedPhrase seed_phrase={seed_phrase.clone()} {cb}/>)
            }
        }
    }
}

#[function_component]
pub fn LoggedOutApp(props: &PageProps) -> Html {
    let login_cb = Callback::from({
        let set_page = props.set_page.clone();
        move |_| set_page.emit(Pages::Unlocked)
    });

    let section = use_state(|| Sections::Login {
        cb: login_cb.clone(),
    });

    let register_cb = Callback::from({
        let login_cb = login_cb.clone();
        let section = section.clone();

        move |seed_phrase: String| {
            section.set(Sections::SeedPhrase {
                seed_phrase,
                cb: login_cb.clone(),
            })
        }
    });

    let tabs = [
        NavTab {
            text: "Login".to_string(),
            section: Sections::Login {
                cb: login_cb.clone(),
            },
            button: None,
        },
        NavTab {
            text: "Register".to_string(),
            section: Sections::Register {
                cb: register_cb.clone(),
            },
            button: None,
        },
    ]
    .to_vec();

    let child = section.render();

    html! {
        <div>
            <Nav<Sections> {section} set_page={props.set_page.clone()} {tabs}/>
            {child}
        </div>
    }
}
