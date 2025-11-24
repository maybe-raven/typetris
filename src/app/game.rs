mod swatch;

use gloo::{events::EventListener, utils::window};
use gloo_console::log;
use gloo_timers::callback::Interval;
use typetris::game::Event;
use typetris::game::Game as GameState;
use typetris::game::board::BoardPosition;
use typetris::game::settings::Settings;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement,
    js_sys::{self},
    wasm_bindgen::{JsCast, UnwrapThrowExt},
};
use yew::prelude::*;

use swatch::Swatch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Msg {
    Tick,
    Keydown(KeyboardEvent),
    NewGame,
}

#[derive(Debug, Clone, Properties, PartialEq, Eq)]
pub(crate) struct Props {}

pub(crate) struct Game {
    _tick_handle: Interval,
    _listener: EventListener,
    state: GameState,
    canvas_node: NodeRef,
    last_timestamp: f64,
    swatch: Swatch,
}

impl Game {
    fn tick(&mut self) -> bool {
        let timestamp = js_sys::Date::new_0().value_of();
        let delta_time = timestamp - self.last_timestamp;
        self.last_timestamp = timestamp;

        self.state
            .handle_event(typetris::game::Event::Tick(delta_time))
    }

    fn keydown(&mut self, event: KeyboardEvent) -> bool {
        let event = match event.key().as_str() {
            "Enter" | "Tab" | " " => Event::Next,
            "ArrowLeft" => Event::Left,
            "h" if event.ctrl_key() => Event::Left,
            "ArrowRight" => Event::Right,
            "l" if event.ctrl_key() => Event::Right,
            "Backspace" => Event::Delete,
            key if key.len() == 1 && key.is_ascii() => Event::Type(key.chars().next().unwrap()),
            _ => return false,
        };
        self.state.handle_event(event)
    }

    fn new_game(&mut self) -> bool {
        self.last_timestamp = js_sys::Date::new_0().value_of();
        self.state.handle_event(Event::NewGame)
    }
}

impl Component for Game {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let _tick_handle = Interval::new(1_000 / 30, move || link.send_message(Msg::Tick));
        let timestamp = js_sys::Date::new_0().value_of();
        let callback = ctx.link().callback(|e: KeyboardEvent| {
            log!("keydown", &e);
            Msg::Keydown(e)
        });
        let _listener = EventListener::new(&window(), "keydown", move |e| {
            callback.emit(e.clone().dyn_into().unwrap_throw())
        });
        Self {
            _tick_handle,
            _listener,
            state: GameState::new(Settings::default().with_starts_with_splash(true)),
            canvas_node: NodeRef::default(),
            last_timestamp: timestamp,
            swatch: Swatch::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = format!(
            "aspect-ratio: {} / {}",
            self.state.board().width(),
            self.state.board().height()
        );
        let new_game_onclick = ctx.link().callback(|_| Msg::NewGame);
        html! {
            <>
                <div
                    class="flex h-screen w-full flex-col items-center justify-center gap-8 overflow-hidden p-4 lg:flex-row"
                >
                    <canvas
                        class="w-full max-w-screen-sm lg:h-full lg:w-auto lg:max-w-none"
                        style={style}
                        ref={self.canvas_node.clone()}
                    />
                    <div class="flex flex-col items-center justify-center">
                        if self.state.is_splash(){
                            <p>{"You have to type each word before you can move it."}</p>
                            <p>{"Line up and fill each row to clear it and score."}</p>
                            <p>
                                <span class="text-primary">{"Left"}</span>{" and "}<span
                                    class="text-primary"
                                    >{"right"}</span
                                >{" arrow keys move the blocks."}
                            </p>
                            <p>
                                <span class="text-primary">{"Enter"}</span>{" and "}<span
                                    class="text-primary"
                                    >{"tab"}</span
                                >{" key drops the block"}
                            </p>
                            <button
                                class="bg-primary bg-base mt-4 max-w-fit rounded-full px-4 py-2 text-sm font-semibold shadow-sm"
                                onclick={new_game_onclick}
                            >
                                {"Play"}
                            </button>
                        } else {
                            if self.state.is_game_over() {
                                <h1 class="text-error text-8xl font-bold">{"Game Over"}</h1>
                            }
                            <h1 class="text-light1 text-6xl font-bold">{"Score:"}</h1>
                            <h2 class="text-light2 text-4xl">{self.state.score()}</h2>
                            <button class="font-semibold text-sm bg-primary rounded-full shadow-sm px-4 py-2 mt-4 max-w-fit bg-base" onclick={new_game_onclick}>{"Restart"}</button>
                        }
                    </div>
                </div>
            </>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        let canvas: HtmlCanvasElement = self.canvas_node.cast().unwrap();
        canvas.focus().unwrap();
        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        let (canvas_height, canvas_width) = if first_render {
            let rect = canvas.get_bounding_client_rect();
            let window = web_sys::window().unwrap();
            self.swatch
                .extract(window.get_computed_style(&canvas).unwrap().unwrap());
            let dpr = window.device_pixel_ratio();
            let canvas_height = rect.height() * dpr;
            let canvas_width = rect.width() * dpr;
            canvas.set_height(canvas_height as _);
            canvas.set_width(canvas_width as _);
            (canvas_height, canvas_width)
        } else {
            (canvas.height() as f64, canvas.width() as f64)
        };

        let cell_width = canvas_width / self.state.board().width() as f64;
        let cell_height = canvas_height / self.state.board().height() as f64;

        context.set_fill_style_str(&self.swatch.bg_color);
        context.fill_rect(0.0, 0.0, canvas_width, canvas_height);
        context.set_font(format!("normal {:.0}px system-ui", cell_width * 0.7).as_str());
        context.set_text_align("center");
        context.set_text_baseline("middle");
        context.set_line_width(5.0);

        for (index, block) in self.state.board().blocks().iter().enumerate() {
            let pos = block.position();
            let x = pos.x as f64 * cell_width;
            let y = pos.y as f64 * cell_height;
            let width = block.width() as f64 * cell_width;

            context.begin_path();
            context.rect(x, y, width, cell_height);
            context.set_fill_style_str(if block.is_interactable() {
                &self.swatch.regular_block_color
            } else {
                &self.swatch.disabled_block_color
            });
            context.fill();

            if self.state.board().get_focused_index() == Some(index) {
                for (i, (a, b)) in block
                    .input_text()
                    .chars()
                    .zip(block.assigned_text().chars())
                    .enumerate()
                {
                    let x = x + i as f64 * cell_width;
                    let color = if a == b {
                        &self.swatch.success_color
                    } else {
                        &self.swatch.error_color
                    };
                    context.set_fill_style_str(color);
                    context.fill_rect(x, y, cell_width, cell_height);
                }
            }

            context.set_fill_style_str("white");
            for i in 0..block.assigned_text().len() {
                let x = x + i as f64 * cell_width + cell_width / 2.0;
                let y = y + cell_height / 2.0;
                context
                    .fill_text(&block.assigned_text()[i..i + 1], x, y)
                    .unwrap();
            }

            for i in 1..block.width() {
                let x = i as f64 * cell_width + x;
                context.move_to(x, y);
                context.line_to(x, y + cell_height);
            }
            context.set_stroke_style_str("black");
            context.stroke();

            if let Some(focus) = self.state.board().get_focused() {
                let n = focus.input_text().len();
                if n < focus.assigned_text().len() {
                    let BoardPosition { x, y } = focus.position();
                    let x = x as f64 * cell_width + n as f64 * cell_width;
                    let y = y as f64 * cell_height;
                    context.begin_path();
                    context
                        .arc(
                            x + cell_width * 0.5,
                            y + cell_height * 0.5,
                            cell_width.min(cell_height) * 0.4,
                            0.0,
                            std::f64::consts::PI * 2.0,
                        )
                        .unwrap();
                    context.move_to(x + cell_width * 0.1, y + cell_height * 0.1);
                    context.line_to(x + cell_width * 0.2, y + cell_height * 0.2);
                    context.move_to(x + cell_width * 0.1, y + cell_height * 0.9);
                    context.line_to(x + cell_width * 0.2, y + cell_height * 0.8);
                    context.move_to(x + cell_width * 0.9, y + cell_height * 0.1);
                    context.line_to(x + cell_width * 0.8, y + cell_height * 0.2);
                    context.move_to(x + cell_width * 0.9, y + cell_height * 0.9);
                    context.line_to(x + cell_width * 0.8, y + cell_height * 0.8);
                    context.set_stroke_style_str(&self.swatch.reticle_color);
                    context.stroke();
                }
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => self.tick(),
            Msg::Keydown(e) => self.keydown(e),
            Msg::NewGame => self.new_game(),
        }
    }
}
