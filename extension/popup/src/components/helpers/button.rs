use web_sys::MouseEvent;
use yew::{classes, function_component, html, Callback, Html, Properties};

#[derive(Clone, PartialEq)]
pub enum ButtonVariants {
    Primary,
    Dark,
}

#[derive(Properties, PartialEq)]
pub struct InputProps {
    pub text: String,
    pub onclick: Callback<MouseEvent>,
    pub class: Option<String>,
    pub variant: Option<ButtonVariants>,
}

#[function_component]
pub fn Button(props: &InputProps) -> Html {
    let variant = props.variant.clone().unwrap_or(ButtonVariants::Primary);
    let class = match variant {
        ButtonVariants::Primary => "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 w-full",
        ButtonVariants::Dark => "text-white bg-gray-700 hover:bg-gray-800 focus:ring-4 focus:ring-gray-300 font-medium rounded-lg text-sm px-5 py-2.5 w-full",
    };

    let class = classes!(class, props.class.clone().unwrap_or("".to_string()));

    let onclick = props.onclick.clone();

    html! {
        <button {onclick} type="button" {class}>{props.text.clone()}</button>
    }
}
