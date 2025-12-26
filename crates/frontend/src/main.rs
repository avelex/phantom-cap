use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
mod router;
use components::*;
use router::route::{Route, switch};

#[component]
fn App() -> Html {
    html! {
        <BrowserRouter>
            <div class="h-screen w-full h-full border-2 border-black flex flex-col">
                <header::Header />
                <Switch<Route> render={switch} />
                //<footer::Footer />
            </div>
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
