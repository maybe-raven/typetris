mod game;

use yew::prelude::*;

use game::Game;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <Game />
    }
}
