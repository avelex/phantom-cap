use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::*;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/cap/:id")]
    UpgradeCap { id: AttrValue },
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <home::HomePage /> },
        Route::UpgradeCap { id } => html! { <upgrade_cap::UpgradeCapPage id={id} /> },
        Route::NotFound => html! { <not_found::NotFoundPage /> },
    }
}
