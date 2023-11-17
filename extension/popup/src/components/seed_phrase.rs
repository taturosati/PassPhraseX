use crate::components::helpers::button::Button;
use yew::{function_component, html, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub seed_phrase: String,
    pub cb: Callback<()>,
}

#[function_component]
pub fn SeedPhrase(props: &Props) -> Html {
    let onclick = {
        let cb = props.cb.clone();

        move |_| {
            cb.emit(());
        }
    };

    html! {
        <div>
            <p>{"Your seed phrase is:"}</p>
            <p><b>{props.seed_phrase.clone()}</b></p>
            <p>{"If you lose your seed phrase, you will lose access to your account, make sure to store it safely"}</p>
            <Button text={"Continue"} {onclick} />
        </div>
    }
}
