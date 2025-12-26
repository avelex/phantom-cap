use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::route::Route;

#[component]
pub fn Header() -> Html {
    html! {
        <header class="border-b-2 border-black px-4 py-3">
            <Link<Route> to={Route::Home} classes="text-2xl font-semibold">{ "Phantom Cap" }</Link<Route>>
        </header>
    }
}
