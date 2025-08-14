pub trait RenderBackend {
    type Color;
    fn get_dimension(&self) -> (f64, f64);
    fn draw_text(&mut self);
    fn draw_rect(&mut self);
}

pub struct Renderer<C> {
    regular_block_color: C,
}

impl<B: RenderBackend> Renderer<B::Color> {
    pub fn render(&self, backend: &B) {
        unimplemented!()
    }
}
