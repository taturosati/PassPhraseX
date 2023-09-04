use crate::pages::locked::LockedApp;
use crate::pages::logged_out::LoggedOutApp;
use crate::pages::unlocked::UnlockedApp;
use yew::{html, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct PageProps {
    pub set_page: Callback<Pages>,
}

#[derive(PartialEq)]
pub enum Pages {
    Login,
    Unlock,
    Unlocked,
    Error,
}

pub trait Render<T: Properties> {
    fn render(&self, props: &T) -> Html;
}

impl Render<PageProps> for Pages {
    fn render(&self, props: &PageProps) -> Html {
        let set_page = props.set_page.clone();
        match self {
            Pages::Login => {
                html!(<LoggedOutApp {set_page}/>)
            }
            Pages::Unlock => {
                html!(<LockedApp {set_page}/>)
            }
            Pages::Unlocked => {
                html!(<UnlockedApp {set_page}/>)
            }
            Pages::Error => {
                html!(<div>{"Unknown Error"}</div>)
            }
        }
    }
}

pub mod locked;
pub mod logged_out;
pub mod unlocked;
