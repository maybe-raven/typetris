use std::iter::FusedIterator;

use rand::{Rng, rng, seq::IndexedRandom};

use super::Direction;

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

    pub(super) fn move_horizontaly(&mut self, direction: Direction, width: usize) {
        if !self.complete {
            return;
        }
        self.x = match direction {
            Direction::Left => self.x.saturating_sub(1),
            Direction::Right => self.max_y(width).max(self.x + 1),
        };
    }

    #[inline]
    pub(super) fn move_vertically(&mut self, dy: f64) {
        self.y += dy;
    }

    #[inline]
    pub(super) fn check_text(&mut self, text: String) {
        if self.text == text {
            self.complete = true;
        }
    }

    #[inline]
    pub(super) fn get_y(&self, max_y: usize) -> usize {
        max_y.min(self.y.floor() as usize)
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
    pub(super) fn chars(&self) -> impl ExactSizeIterator<Item = &'static str> + FusedIterator {
        assert!(self.text.is_ascii());
        (0..self.text.len()).map(|i| &self.text[i..i + 1])
    }
}
