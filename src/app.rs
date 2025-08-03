use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main>
            <img
                class="h-[20em] text-center font-sans text-[#fff6d5]"
                src="https://yew.rs/img/logo.svg"
                alt="Yew logo"
            />
            <h1>{ "Hello World!" }</h1>
            <span class="mt-[-1em] block">
                { "from Yew with " }<i class="text-[1.75em]">{ "❤️" }</i>
            </span>
        </main>
    }
}
