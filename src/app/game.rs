mod swatch;

use gloo::{events::EventListener, utils::window};
use gloo_console::log;
use gloo_timers::callback::Interval;
use typetris::game::Event;
use typetris::game::Game as GameState;
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
        if self.state.game_over() {
            return false;
        }

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
            state: GameState::default(),
            canvas_node: NodeRef::default(),
            last_timestamp: timestamp,
            swatch: Swatch::new(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <canvas class="h-screen aspect-[13/15]" ref={self.canvas_node.clone()} />
                if self.state.game_over() {
                    <h1 class="text-error text-9xl font-bold">{"Game Over"}</h1>
                } else {
                    <div class="flex flex-col items-center justify-center">
                        <h1 class="text-9xl font-bold text-light1">{"Score:"}</h1>
                        <h2 class="text-7xl text-light2">{self.state.score()}</h2>
                    </div>
                }
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
        context.set_stroke_style_str("black");

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
            context.stroke();
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => self.tick(),
            Msg::Keydown(e) => self.keydown(e),
        }
    }
}
