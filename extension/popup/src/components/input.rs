use material_yew::MatTextField;
use yew::{function_component, html, Callback, Html, Properties, UseStateHandle};

#[derive(Properties, PartialEq)]
pub struct InputProps {
    pub label: String,
    pub value: UseStateHandle<String>,
}

#[function_component]
pub fn Input(props: &InputProps) -> Html {
    let on_input = {
        let value = props.value.clone();
        Callback::from(move |val: String| {
            value.set(val.clone());
        })
    };

    html! {
        <MatTextField outlined=true label={props.label.clone()} value={(*props.value).clone()} oninput={on_input} />
    }
}
