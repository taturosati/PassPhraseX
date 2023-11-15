use crate::api::app_request;
use crate::pages::Pages;
use gloo_console as console;
use messages::AppRequestPayload;
use yew::{
    classes, function_component, html, use_effect_with_deps, use_state, Callback, Html, Properties,
    UseStateHandle,
};

#[derive(Properties, PartialEq, Clone)]
pub struct NavTab<Section: PartialEq + Clone> {
    pub text: String,
    pub section: Section,
    pub button: Option<NavTabButtonProps>,
}

#[derive(Properties, PartialEq, Clone)]
pub struct NavTabButtonProps {
    pub set_page: Callback<Pages>,
    pub page: Pages,
    pub payload: AppRequestPayload,
}

impl<Section: PartialEq + Clone + 'static> NavTab<Section> {
    pub fn render(&self, section: UseStateHandle<Section>) -> Html {
        if let Some(button) = self.button.clone() {
            html! {
                <TabButton text={self.text.clone()} set_page={button.set_page} page={button.page} payload={button.payload} />
            }
        } else {
            html! {
                <Tab<Section> text={self.text.clone()} this_section={self.section.clone()} {section} />
            }
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct NavProps<Section: PartialEq + Clone> {
    pub section: UseStateHandle<Section>,
    pub set_page: Callback<Pages>,
    pub tabs: Vec<NavTab<Section>>,
}

#[function_component]
pub fn Nav<Section: PartialEq + Clone + 'static>(props: &NavProps<Section>) -> Html {
    let section = props.section.clone();

    html! {
        <ul class={classes!("mb-2", "flex", "list-none", "flex-row", "flex-wrap", "border-b-0", "pl-0")} role="tablist">
            {props.tabs.iter().map(|tab| tab.render(section.clone())).collect::<Html>()}
        </ul>
    }
}

#[derive(Properties, PartialEq)]
struct TabProps<Section: PartialEq> {
    pub section: UseStateHandle<Section>,
    pub this_section: Section,
    pub text: String,
}

#[function_component]
fn Tab<Section: PartialEq + Clone + 'static>(props: &TabProps<Section>) -> Html {
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
                if variant_eq((*section).clone(), this_section.clone()) {
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

#[derive(Properties, PartialEq)]
struct TabButtonProps {
    pub text: String,
    pub set_page: Callback<Pages>,
    pub page: Pages,
    pub payload: AppRequestPayload,
}

#[function_component]
fn TabButton(props: &TabButtonProps) -> Html {
    let onclick = {
        let set_page = props.set_page.clone();
        let page = props.page.clone();
        let payload = props.payload.clone();

        move |_| {
            let set_page = set_page.clone();
            let page = page.clone();
            let payload = payload.clone();

            app_request(payload, move |res| match res {
                Ok(_) => {
                    set_page.emit(page.clone());
                }
                Err(err) => {
                    console::error!("Error: {:?}", err);
                }
            });
        }
    };

    html! {
        <li role="presentation" class="flex-grow basis-0 text-center">
            <a {onclick} role="tab" class={"mb-2 block border-x-0 border-b-2 border-t-0 px-7 pb-3.5 pt-4 text-xs font-medium uppercase leading-tight hover:isolate hover:bg-neutral-100 cursor-pointer border-transparent text-neutral-500 focus:isolate focus:border-transparent"}
            >{props.text.clone()}</a>
        </li>
    }
}

fn variant_eq<T>(a: T, b: T) -> bool {
    std::mem::discriminant(&a) == std::mem::discriminant(&b)
}
