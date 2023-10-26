use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::html::onchange::Event;
use yew::{classes, function_component, html, Callback, Html, Properties, UseStateHandle};

#[derive(Properties, PartialEq)]
pub struct InputProps {
    pub input_type: Option<String>,
    pub label: String,
    pub value: UseStateHandle<String>,
    pub error: Option<String>,
}

#[function_component]
pub fn Input(props: &InputProps) -> Html {
    let input_type = props.input_type.clone().unwrap_or("text".to_string());
    let value = props.value.clone();

    let onchange: Callback<Event> = {
        let value = value.clone();
        Callback::from(move |e: Event| {
            let target: EventTarget = e
                .target()
                .expect("Event should have a target when dispatched");
            value.set(target.unchecked_into::<HtmlInputElement>().value());
        })
    };

    let border_color = props
        .error
        .clone()
        .map(|_| "border-red-500")
        .unwrap_or("border-gray-300");

    html! {
        <div class={classes!("mb-2")}>
            <label class={classes!("block", "mb-1", "text-sm", "font-medium")}>{props.label.clone()}</label>
            <input onchange={onchange} type={input_type}
                class={classes!("block", "w-full", "p-2", "border", border_color, "rounded-lg", "sm:text-xs")}
                value={(*props.value).clone()}/>
            { props.error.clone().map(|error| html! { <p class={classes!("text-red-500", "text-xs")}>{ error }</p> }) }
        </div>
    }
}
