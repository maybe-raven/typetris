mod block;

use std::cmp::Ordering;

use block::Block;
use gloo::events::EventListener;
use gloo::utils::window;
use gloo_console::log;
use gloo_timers::callback::Interval;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement,
    js_sys::{self},
    wasm_bindgen::{JsCast, UnwrapThrowExt},
};
use yew::prelude::*;

const BLOCK_NORMAL_FALL_SPEED: f64 = 2.0;
const BLOCK_COMPLETED_FALL_SPEED: f64 = 10.0;
const SPAWN_DELAY: f64 = 3.0;
const BOARD_WIDTH: usize = 12;
const BOARD_HEIGHT: usize = 15;

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
    canvas_node: NodeRef,
    blocks: Vec<Block>,
    last_timestamp: f64,
    spawn_timer: f64,
    text: String,
    focus_index: usize,
    moving_index: usize,
    game_over: bool,
}

impl Game {
    fn tick(&mut self) -> bool {
        if self.game_over {
            return false;
        }

        let timestamp = js_sys::Date::new_0().value_of();
        let delta_time = (timestamp - self.last_timestamp) / 1_000.0;
        self.last_timestamp = timestamp;

        let mut ret = self.moving_index < self.blocks.len();

        self.spawn_timer -= delta_time;
        if self.spawn_timer <= 0.0 {
            self.spawn_timer += SPAWN_DELAY;
            self.blocks.push(Block::new(BOARD_WIDTH));
            ret = true;
        }

        fn find_max_y(blocks: &[Block], i: usize) -> usize {
            let block = &blocks[i];
            let y = block.get_y();
            let x0 = block.get_x();
            let x1 = x0 + block.width() - 1;
            blocks
                .iter()
                .enumerate()
                .filter_map(|(j, block)| {
                    let bx0 = block.get_x();
                    let bx1 = bx0 + block.width() - 1;
                    (j != i && block.get_y() > y && !(bx1 < x0 || bx0 > x1))
                        .then_some(block.get_y())
                })
                .min()
                .unwrap_or(BOARD_HEIGHT)
                - 1
        }

        let mut newly_rested = false;
        for i in self.moving_index..self.blocks.len() {
            let max_y = find_max_y(&self.blocks, i);
            let speed = if i < self.focus_index {
                BLOCK_COMPLETED_FALL_SPEED
            } else {
                BLOCK_NORMAL_FALL_SPEED
            };
            let rested = self.blocks[i].move_vertically(delta_time * speed, max_y as f64);

            if rested {
                newly_rested = true;
                self.moving_index = i + 1;
                if self.moving_index > self.focus_index {
                    self.focus_index = self.moving_index;
                    self.text.clear();
                }
                if max_y == 0 {
                    self.game_over = true;
                }
            }
        }

        if newly_rested {
            let mut rested: Vec<(usize, &Block)> = self.blocks[..self.moving_index]
                .iter()
                .enumerate()
                .collect();
            rested.sort_by(|a, b| match a.1.get_y().cmp(&b.1.get_y()) {
                Ordering::Equal => a.1.get_x().cmp(&b.1.get_x()),
                o => o,
            });

            let mut removals = Vec::new();
            for chunk in rested.chunk_by(|a, b| a.1.get_y() == b.1.get_y()) {
                let mut first = true;
                let mut last = 0;
                for window in chunk.windows(2) {
                    let &[(_, a), (_, b)] = window else {
                        unreachable!();
                    };
                    let ax0 = a.get_x();
                    let ax1 = ax0 + a.width();
                    let bx0 = b.get_x();
                    let bx1 = bx0 + b.width();

                    if first {
                        if ax0 != 0 {
                            continue;
                        }
                        first = false;
                    }
                    if ax1 != bx0 {
                        continue;
                    }
                    last = bx1;
                }
                if last != BOARD_WIDTH {
                    continue;
                }

                removals.extend(chunk.iter().map(|(i, _)| *i));
            }

            self.focus_index -= removals.len();
            self.moving_index -= removals.len();
            removals.sort();
            for i in removals.into_iter().rev() {
                self.blocks.remove(i);
            }
            for i in 0..self.moving_index {
                let max_y = find_max_y(&self.blocks, i);
                self.blocks[i].move_vertically(BOARD_HEIGHT as _, max_y as _);
            }
        }

        ret
    }

    fn keydown(&mut self, event: KeyboardEvent) -> bool {
        if self.blocks.len() <= self.focus_index {
            return false;
        }

        match event.key().as_str() {
            "Enter" | "Tab" | " " => {
                self.focus_index += 1;
                self.text.clear();
            }
            "ArrowLeft" => self.left(),
            "h" if event.ctrl_key() => self.left(),
            "ArrowRight" => self.right(),
            "l" if event.ctrl_key() => self.right(),
            "Backspace" => {
                self.text.pop();
            }
            key if key.len() == 1 => {
                self.text.push_str(key);
                self.blocks[self.focus_index].check_text(&self.text);
            }
            _ => (),
        }
        false
    }

    fn left(&mut self) {
        let block = &self.blocks[self.focus_index];
        let x = block.get_x();
        let y = self.blocks[self.focus_index].get_y();
        for block in self.blocks.iter().take(self.focus_index) {
            if block.get_y() == y && block.get_x() + block.width() == x {
                return;
            }
        }
        self.blocks[self.focus_index].move_left();
    }

    fn right(&mut self) {
        let block = &self.blocks[self.focus_index];
        let x = block.get_x() + block.width();
        let y = self.blocks[self.focus_index].get_y();
        for block in self.blocks.iter().take(self.focus_index) {
            if block.get_y() == y && block.get_x() == x {
                return;
            }
        }
        self.blocks[self.focus_index].move_right(BOARD_WIDTH);
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
            canvas_node: NodeRef::default(),
            blocks: Vec::new(),
            last_timestamp: timestamp,
            spawn_timer: SPAWN_DELAY,
            text: String::new(),
            focus_index: 0,
            moving_index: 0,
            game_over: false,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <canvas class="h-screen aspect-[12/15] bg-fore" ref={self.canvas_node.clone()} />
                if self.game_over {
                    <h1 class="text-red text-9xl font-bold">{"Game Over"}</h1>
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
            let dpr = web_sys::window().unwrap().device_pixel_ratio();
            let canvas_height = rect.height() * dpr;
            let canvas_width = rect.width() * dpr;
            canvas.set_height(canvas_height as _);
            canvas.set_width(canvas_width as _);
            (canvas_height, canvas_width)
        } else {
            (canvas.height() as f64, canvas.width() as f64)
        };

        let cell_width = canvas_width / BOARD_WIDTH as f64;
        let cell_height = canvas_height / BOARD_HEIGHT as f64;

        context.clear_rect(0.0, 0.0, canvas_width, canvas_height);
        context.set_font(format!("normal {:.0}px system-ui", cell_width * 0.7).as_str());
        context.set_text_align("center");
        context.set_text_baseline("middle");
        context.set_stroke_style_str("black");

        for (index, block) in self.blocks.iter().enumerate() {
            let x = block.get_x() as f64 * cell_width;
            let y = block.get_y() as f64 * cell_height;
            let width = block.width() as f64 * cell_width;

            context.begin_path();
            context.rect(x, y, width, cell_height);
            context.set_fill_style_str("blue");
            context.fill();

            if index == self.focus_index {
                for (i, (a, b)) in self.text.chars().zip(block.text().chars()).enumerate() {
                    let x = x + i as f64 * cell_width;
                    let color = if a == b { "green" } else { "red" };
                    context.set_fill_style_str(color);
                    context.fill_rect(x, y, cell_width, cell_height);
                }
            }

            for i in 1..block.width() {
                let x = i as f64 * cell_width + x;
                context.move_to(x, y);
                context.line_to(x, y + cell_height);
            }
            context.stroke();

            context.set_fill_style_str("black");
            for (i, char) in (0..block.text().len()).map(|i| (i, &block.text()[i..i + 1])) {
                let x = x + i as f64 * cell_width + cell_width / 2.0;
                let y = y + cell_height / 2.0;
                context.fill_text(char, x, y).unwrap();
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => self.tick(),
            Msg::Keydown(e) => self.keydown(e),
        }
    }
}
