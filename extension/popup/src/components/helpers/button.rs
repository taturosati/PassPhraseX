use web_sys::MouseEvent;
use yew::{classes, function_component, html, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct InputProps {
    pub text: String,
    pub onclick: Callback<MouseEvent>,
}

#[function_component]
pub fn Button(props: &InputProps) -> Html {
    let onclick = props.onclick.clone();
    html! {
        <button {onclick} type="button" class={classes!("text-white", "bg-blue-700", "hover:bg-blue-800", "focus:ring-4", "focus:ring-blue-300", "font-medium", "rounded-lg", "text-sm", "px-5", "py-2.5", "w-full")}>{props.text.clone()}</button>
    }
}
