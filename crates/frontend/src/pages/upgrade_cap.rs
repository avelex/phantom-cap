use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct UpgradeCapProps {
    pub id: AttrValue,
}

#[component]
pub fn UpgradeCapPage(&UpgradeCapProps { ref id }: &UpgradeCapProps) -> Html {
    html! {
        <h1>{ format!("Upgrade Cap: {}", id) }</h1>
    }
}
