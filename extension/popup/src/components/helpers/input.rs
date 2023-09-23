use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::html::onchange::Event;
use yew::{classes, function_component, html, Callback, Html, Properties, UseStateHandle};

#[derive(Properties, PartialEq)]
pub struct InputProps {
    pub label: String,
    pub value: UseStateHandle<String>,
}

#[function_component]
pub fn Input(props: &InputProps) -> Html {
    // let input_ref = use_node_ref();
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

    // let onchange = Callback::from(move |e: Event| {
    //     let target: EventTarget = e
    //         .target()
    //         .expect("Event should have a target when dispatched");
    //
    //     // value.set(target.unchecked_into::<HtmlInputElement>().value());
    // });

    html! {
        <div class={classes!("mb-2")}>
            <label class={classes!("block", "mb-1", "text-sm", "font-medium")}>{props.label.clone()}</label>
            <input onchange={onchange} type="text"
                class={classes!("block", "w-full", "p-2", "border", "border-gray-300", "rounded-lg", "sm:text-xs")}
                value={(*props.value).clone()}/>
        </div>
    }
}
