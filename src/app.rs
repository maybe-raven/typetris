use yew::prelude::*;

use crate::game::Game;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <Game />
    }
}
