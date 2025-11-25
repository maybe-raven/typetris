pub mod bounding_box;

use getset::{CopyGetters, Getters};
use rand::{Rng, rng, seq::IteratorRandom};
use vector2d::Vector2D;

use crate::game::{board::BoardPosition, english::WORDS};
use bounding_box::{BoundingBox, MixAdd};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Settled,
    Falling,
    Interactable,
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct Block {
    #[getset(get_copy = "pub")]
    pub(super) state: State,
    #[getset(get = "pub")]
    cells: Vec<BlockCell>,
    #[getset(get_copy = "pub")]
    bounding_box: BoundingBox,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Getters, CopyGetters)]
pub struct BlockCell {
    pub assigned_char: u8,
    pub input_char: Option<u8>,
    pub position: BoardPosition,
}

impl BlockCell {
    pub const fn new(assigned_char: u8, position: BoardPosition) -> Option<Self> {
        if !assigned_char.is_ascii() {
            return None;
        }
        Some(Self {
            assigned_char: assigned_char as u8,
            input_char: None,
            position,
        })
    }
    pub const unsafe fn new_unchecked(assigned_char: u8, position: BoardPosition) -> Self {
        Self {
            assigned_char,
            input_char: None,
            position,
        }
    }
    #[inline]
    pub const fn is_correct(&self) -> bool {
        let Some(ic) = self.input_char else {
            return false;
        };
        ic == self.assigned_char
    }
}

impl Block {
    pub fn random(board_width: u8) -> Self {
        let mut rng = rng();
        let text = *WORDS
            .iter()
            .filter(|w| w.len() <= (board_width as usize))
            .choose(&mut rng)
            .unwrap();
        let x = rng.random_range(0..=(board_width - text.len() as u8));
        Self::new_line(text, State::Interactable, x, 0)
            .expect("default dictionay should only contain ascii words")
    }

    #[inline]
    pub(super) fn new_line(
        assigned_text: &'static str,
        state: State,
        x: u8,
        y: u8,
    ) -> Option<Self> {
        if assigned_text.is_ascii() {
            Some(Self {
                state,
                cells: assigned_text
                    .bytes()
                    .enumerate()
                    .map(|(i, c)| unsafe {
                        // `assigned_text` is just confirmed to be ascii in the outer scope
                        BlockCell::new_unchecked(c, BoardPosition(Vector2D { x: x + i as u8, y }))
                    })
                    .collect(),
                bounding_box: BoundingBox {
                    x,
                    y,
                    width: assigned_text.len() as u8,
                    height: 1,
                },
            })
        } else {
            None
        }
    }

    // #[inline]
    // pub(super) fn new(state: State, cells: Vec<BlockCell>) -> Self {
    //     Self { state, cells }
    // }

    #[inline]
    pub fn x(&self) -> u8 {
        self.bounding_box.x
    }

    #[inline]
    pub fn y(&self) -> u8 {
        self.bounding_box.y
    }

    #[inline]
    pub fn is_correct(&self) -> bool {
        self.cells.iter().all(|c| c.is_correct())
    }

    pub(super) fn add_char(&mut self, ch: char) -> bool {
        if ch.is_ascii_alphabetic() && self.is_interactable() {
            for cell in self.cells.iter_mut() {
                if cell.input_char.is_none() {
                    cell.input_char = Some(ch as u8);
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    pub(super) fn delete_char(&mut self) -> bool {
        if self.is_interactable() {
            for cell in self.cells.iter_mut().rev() {
                if cell.input_char.is_some() {
                    cell.input_char = None;
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    #[inline]
    pub fn is_settled(&self) -> bool {
        matches!(self.state, State::Settled)
    }

    #[inline]
    pub fn is_interactable(&self) -> bool {
        matches!(self.state, State::Interactable)
    }

    #[inline]
    pub fn is_falling(&self) -> bool {
        matches!(self.state, State::Falling)
    }

    #[inline]
    pub fn is_movable(&self) -> bool {
        self.is_interactable() && self.is_correct()
    }

    #[inline]
    pub(super) fn shift(&mut self, offset: Vector2D<i8>) {
        self.bounding_box.shift(offset);
        for cell in &mut self.cells {
            cell.position.mix_add_assign(offset);
        }
    }

    #[inline]
    pub(super) fn intersect(&self, other: &Self) -> bool {
        // check bounding box intersection first because it's faster,
        // then check each individual cells
        self.bounding_box.intersect(other.bounding_box)
            && self
                .cells
                .iter()
                .any(|a| other.cells.iter().any(|b| a.position == b.position))
    }
}

#[cfg(test)]
impl Block {
    #[inline]
    pub(super) fn with_text_x(assigned_text: &'static str, x: u8) -> Self {
        Self::new_line(assigned_text, State::Interactable, x, 0)
            .expect("`assigned_text` should be ascii")
    }

    #[inline]
    pub(super) fn new_interactable(assigned_text: &'static str, x: u8, y: u8) -> Self {
        Self::new_line(assigned_text, State::Interactable, x, y)
            .expect("`assigned_text` should be ascii")
    }

    #[inline]
    pub(super) fn new_settled(assigned_text: &'static str, x: u8, y: u8) -> Self {
        Self::new_line(assigned_text, State::Settled, x, y)
            .expect("`assigned_text` should be ascii")
    }

    #[inline]
    pub(super) fn new_falling(assigned_text: &'static str, x: u8, y: u8) -> Self {
        Self::new_line(assigned_text, State::Falling, x, y)
            .expect("`assigned_text` should be ascii")
    }

    #[inline]
    pub(super) fn check_input_text(&self, expected_text: &str) -> bool {
        self.cells
            .iter()
            .zip(expected_text.bytes())
            .all(|(&cell, exp)| cell.input_char == Some(exp))
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU8;

    use iter_tools::Itertools;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn random(width: NonZeroU8) -> bool {
        let b = Block::random(width.get());
        assert!(!b.cells.is_empty());
        assert!(b.cells.is_sorted_by(|a, b| a.position <= b.position));
        assert!(b.cells.iter().map(|b| b.position).all_unique());
        assert!(b.bounding_box.width <= width.get());
        assert_eq!(b.bounding_box.y, 0);
        assert_eq!(
            b.bounding_box.x,
            b.cells.iter().map(|c| c.position.x).min().unwrap()
        );
        assert_eq!(
            b.bounding_box.y,
            b.cells.iter().map(|c| c.position.y).min().unwrap()
        );
        assert_eq!(
            b.bounding_box.x + b.bounding_box.width,
            b.cells.iter().map(|c| c.position.x).max().unwrap()
        );
        assert_eq!(
            b.bounding_box.y + b.bounding_box.height,
            b.cells.iter().map(|c| c.position.y).max().unwrap()
        );
        assert!(
            b.cells
                .iter()
                .all(|c| b.bounding_box.contains(c.position.0))
        );
        assert!(b.cells.iter().all(|c| c.input_char.is_none()));
        assert_eq!(b.state, State::Interactable);
        true
    }

    mod add_del_char {
        use super::*;

        #[inline]
        fn make_block(text: &'static str) -> Block {
            Block::new_line(text, State::Interactable, 0, 0).expect("text should be ascii only")
        }

        fn add_and_assert(
            b: &mut Block,
            ch: char,
            has_update: bool,
            is_correct: bool,
            expected_text: &str,
        ) {
            assert_eq!(b.add_char(ch), has_update);
            assert_eq!(b.state, State::Interactable);
            assert_eq!(b.is_correct(), is_correct);
            assert!(b.check_input_text(expected_text));
        }

        fn delete_and_assert(
            b: &mut Block,
            has_update: bool,
            is_correct: bool,
            expected_text: &str,
        ) {
            assert_eq!(b.delete_char(), has_update);
            assert_eq!(b.state, State::Interactable);
            assert_eq!(b.is_correct(), is_correct);
            assert!(b.check_input_text(expected_text));
        }

        #[test]
        fn invalid_chars() {
            let mut b = make_block("unicode");
            add_and_assert(&mut b, '.', false, false, "");
            add_and_assert(&mut b, '%', false, false, "");
            add_and_assert(&mut b, ' ', false, false, "");
            add_and_assert(&mut b, '"', false, false, "");
            add_and_assert(&mut b, '0', false, false, "");
        }

        #[test]
        fn t0() {
            let mut b = make_block("a");
            add_and_assert(&mut b, 'a', true, true, "a");
            add_and_assert(&mut b, 'b', false, true, "a");
            delete_and_assert(&mut b, true, false, "");
            add_and_assert(&mut b, 'B', true, false, "B");
        }

        #[test]
        fn t1() {
            let mut b = make_block("a");
            add_and_assert(&mut b, 'b', true, false, "b");
            add_and_assert(&mut b, 'c', false, false, "b");
            delete_and_assert(&mut b, true, false, "");
            add_and_assert(&mut b, 'a', true, true, "a");
        }

        #[test]
        fn t2() {
            let mut b = make_block("abc");
            add_and_assert(&mut b, 'a', true, false, "a");
            add_and_assert(&mut b, 'b', true, false, "ab");
            add_and_assert(&mut b, 'C', true, false, "abC");
            add_and_assert(&mut b, 'd', false, false, "abC");
            delete_and_assert(&mut b, true, false, "ab");
            add_and_assert(&mut b, 'c', true, true, "abc");
            delete_and_assert(&mut b, true, false, "ab");
            delete_and_assert(&mut b, true, false, "a");
            delete_and_assert(&mut b, true, false, "");
            delete_and_assert(&mut b, false, false, "");
        }

        #[test]
        fn t3() {
            for state in [State::Falling, State::Settled] {
                let a = Block::new_line("a", state, 0, 0).expect("ascii text should be valid");
                let mut b = a.clone();
                assert!(!b.add_char('a'));
                assert_eq!(a, b);
            }
        }

        #[test]
        fn t4() {
            for state in [State::Falling, State::Settled] {
                let a = Block::new_line("abc", state, 0, 0).expect("ascii text should be valid");
                let mut b = a.clone();
                assert!(!b.delete_char());
                assert_eq!(a, b);
            }
        }
    }
}
