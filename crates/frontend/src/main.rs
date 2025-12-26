use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
use components::*;
use pages::*;

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/cap/:id")]
    UpgradeCap { id: AttrValue },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <home::HomePage /> },
        Route::UpgradeCap { id } => html! { <upgrade_cap::UpgradeCapPage id={id} /> },
        Route::NotFound => html! { <not_found::NotFoundPage /> },
    }
}

#[component]
fn App() -> Html {
    html! {
        <BrowserRouter>
            <div class="flex flex-col h-screen">
                <header::Header />
                <main>
                    <Switch<Route> render={switch} />
                </main>
                <footer::Footer />
            </div>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
