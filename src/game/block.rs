use rand::{Rng, rng, seq::IndexedRandom};

const WORDS: &[&str] = include!("english.in");

#[derive(Debug, Clone, Copy)]
pub(super) struct Block {
    complete: bool,
    x: usize,
    y: f64,
    text: &'static str,
}

#[allow(unused)]
impl Block {
    pub(super) fn new(width: usize) -> Self {
        let mut rng = rng();
        let text = *WORDS.choose(&mut rng).unwrap();
        let mut ret = Self {
            x: 0,
            y: 0.0,
            text,
            complete: false,
        };
        ret.x = rng.random_range(0..=ret.max_y(width));
        ret
    }

    #[inline]
    pub(super) fn move_right(&mut self, width: usize) {
        if self.complete {
            self.x = self.max_y(width).min(self.x + 1);
        }
    }

    #[inline]
    pub(super) fn move_left(&mut self) {
        if self.complete {
            self.x = self.x.saturating_sub(1);
        }
    }

    #[inline]
    pub(super) fn move_vertically(&mut self, dy: f64, max_y: f64) {
        self.y = max_y.min(self.y + dy);
    }

    #[inline]
    pub(super) fn check_text(&mut self, text: &str) {
        if self.text == text {
            self.complete = true;
        }
    }

    #[inline]
    pub(super) fn get_y(&self) -> usize {
        self.y.floor() as usize
    }

    #[inline]
    pub(super) fn get_x(&self) -> usize {
        self.x
    }

    #[inline]
    pub(super) fn width(&self) -> usize {
        self.text.len()
    }

    #[inline]
    fn max_y(&self, width: usize) -> usize {
        width - self.width()
    }

    #[inline]
    pub(super) fn text(&self) -> &'static str {
        self.text
    }
}
