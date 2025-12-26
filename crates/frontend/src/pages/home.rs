use crate::router::route::Route;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[component]
pub fn HomePage() -> Html {
    let navigator = use_navigator().unwrap();
    let input_value = use_state(|| String::new());

    let on_input = {
        let input_value = input_value.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input_value.set(input.value());
        })
    };

    let on_submit = {
        let input_value = input_value.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let value = (*input_value).clone();

            // TODO: add sui object id validation.
            if !value.is_empty() {
                navigator.push(&Route::UpgradeCap { id: value.into() });
            }
        })
    };

    html! {
        <div class="flex-1 flex flex-col items-center justify-center px-4">
            <h2 class="text-2xl font-medium mb-6">{"Caps are now revealed!"}</h2>
            <form onsubmit={on_submit} class="w-full max-w-md">
                <input
                    type="text"
                    placeholder="Search by Upgrade Cap"
                    class="w-full border-2 border-black rounded-full px-6 py-3 text-lg text-center focus:outline-none focus:ring-1 focus:ring-black"
                    oninput={on_input}
                />
            </form>
        </div>
    }
}
