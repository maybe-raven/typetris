use getset::{CopyGetters, Getters};
use rand::{Rng, rng, seq::IteratorRandom};

use crate::game::board::BoardPosition;

include! { "english.rs" }

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
    assigned_text: &'static str,
    #[getset(get = "pub")]
    input_text: String,
    #[getset(get_copy = "pub")]
    pub(super) position: BoardPosition,
}

impl Block {
    pub fn new(board_width: u8) -> Self {
        let mut rng = rng();
        let text = *WORDS
            .iter()
            .filter(|w| w.len() <= (board_width as usize))
            .choose(&mut rng)
            .unwrap();
        let mut ret = Self {
            state: State::Interactable,
            assigned_text: text,
            input_text: String::new(),
            position: BoardPosition::new(),
        };
        ret.position.x = rng.random_range(0..=(board_width - ret.width()));
        ret
    }

    #[inline]
    pub fn is_correct(&self) -> bool {
        self.assigned_text == self.input_text
    }

    #[inline]
    pub(super) fn add_char(&mut self, ch: char) -> bool {
        if ch.is_ascii_alphabetic()
            && self.is_interactable()
            && self.input_text.len() < self.assigned_text.len()
        {
            self.input_text.push(ch);
            true
        } else {
            false
        }
    }

    #[inline]
    pub(super) fn delete_char(&mut self) -> bool {
        if self.is_interactable() {
            self.input_text.pop().is_some()
        } else {
            false
        }
    }

    #[inline]
    pub fn width(&self) -> u8 {
        self.assigned_text.len() as u8
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
    pub fn is_movable(&self) -> bool {
        self.is_interactable() && self.is_correct()
    }

    #[inline]
    pub(super) fn intersect_x(&self, other: &Self) -> bool {
        let ax0 = self.position.x;
        let ax1 = ax0 + self.width() - 1;
        let bx0 = other.position.x;
        let bx1 = bx0 + other.width() - 1;
        !(ax1 < bx0 || bx1 < ax0)
    }
}

#[cfg(test)]
impl Block {
    #[inline]
    pub(super) fn with_text_x(assigned_text: &'static str, x: u8) -> Self {
        Self {
            state: State::Interactable,
            assigned_text,
            input_text: String::new(),
            position: BoardPosition { x, y: 0 },
        }
    }

    #[inline]
    pub(super) fn new_interactable(assigned_text: &'static str, x: u8, y: u8) -> Self {
        Self {
            state: State::Interactable,
            assigned_text,
            input_text: String::new(),
            position: BoardPosition { x, y },
        }
    }

    #[inline]
    pub(super) fn new_settled(assigned_text: &'static str, x: u8, y: u8) -> Self {
        Self {
            state: State::Settled,
            assigned_text,
            input_text: String::new(),
            position: BoardPosition { x, y },
        }
    }

    #[inline]
    pub(super) fn new_falling(assigned_text: &'static str, x: u8, y: u8) -> Self {
        Self {
            state: State::Falling,
            assigned_text,
            input_text: String::new(),
            position: BoardPosition { x, y },
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn new(width: u8) -> bool {
        if width == 0 {
            return true;
        }
        let b = Block::new(width);
        assert!(!b.assigned_text.is_empty());
        assert!(b.assigned_text.is_ascii());
        assert!(b.assigned_text.len() <= width as usize);
        assert!(b.input_text.is_empty());
        assert_eq!(b.position.y, 0);
        assert_eq!(b.state, State::Interactable);
        assert!(b.position.x + b.width() <= width);
        true
    }

    #[test]
    fn intersect() {
        assert!(Block::with_text_x("abc", 0).intersect_x(&Block::with_text_x("abc", 0)));
        assert!(Block::with_text_x("abc", 1).intersect_x(&Block::with_text_x("abc", 0)));
        assert!(Block::with_text_x("abc", 0).intersect_x(&Block::with_text_x("abc", 1)));
        assert!(Block::with_text_x("abc", 2).intersect_x(&Block::with_text_x("abc", 0)));
        assert!(Block::with_text_x("abc", 0).intersect_x(&Block::with_text_x("abc", 2)));
        assert!(Block::with_text_x("abcdef", 0).intersect_x(&Block::with_text_x("abc", 1)));
        assert!(Block::with_text_x("abc", 2).intersect_x(&Block::with_text_x("abcdefg", 0)));
        assert!(!Block::with_text_x("abc", 0).intersect_x(&Block::with_text_x("abc", 3)));
        assert!(!Block::with_text_x("abc", 0).intersect_x(&Block::with_text_x("abcdefg", 6)));
        assert!(!Block::with_text_x("abc", 3).intersect_x(&Block::with_text_x("abc", 0)));
        assert!(!Block::with_text_x("abcdef", 10).intersect_x(&Block::with_text_x("abcdefg", 42)));
        assert!(!Block::with_text_x("abcdef", 100).intersect_x(&Block::with_text_x("abcdefg", 42)));
        assert!(!Block::with_text_x("a", 100).intersect_x(&Block::with_text_x("b", 101)));
        assert!(Block::with_text_x("a", 100).intersect_x(&Block::with_text_x("b", 100)));
    }

    mod add_del_char {
        use super::*;

        #[inline]
        fn make_block(text: &'static str, state: State) -> Block {
            Block {
                state,
                assigned_text: text,
                input_text: String::new(),
                position: BoardPosition { x: 0, y: 0 },
            }
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
            assert_eq!(b.input_text, expected_text);
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
            assert_eq!(b.input_text, expected_text);
        }

        #[test]
        fn invalid_chars() {
            let mut b = make_block("unicode", State::Interactable);
            add_and_assert(&mut b, '.', false, false, "");
            add_and_assert(&mut b, '%', false, false, "");
            add_and_assert(&mut b, ' ', false, false, "");
            add_and_assert(&mut b, '"', false, false, "");
            add_and_assert(&mut b, '0', false, false, "");
        }

        #[test]
        fn t0() {
            let mut b = make_block("a", State::Interactable);
            add_and_assert(&mut b, 'a', true, true, "a");
            add_and_assert(&mut b, 'b', false, true, "a");
            delete_and_assert(&mut b, true, false, "");
            add_and_assert(&mut b, 'B', true, false, "B");
        }

        #[test]
        fn t1() {
            let mut b = make_block("a", State::Interactable);
            add_and_assert(&mut b, 'b', true, false, "b");
            add_and_assert(&mut b, 'c', false, false, "b");
            delete_and_assert(&mut b, true, false, "");
            add_and_assert(&mut b, 'a', true, true, "a");
        }

        #[test]
        fn t2() {
            let mut b = make_block("abc", State::Interactable);
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
                let a = make_block("a", state);
                let mut b = a.clone();
                assert!(!b.add_char('a'));
                assert_eq!(a, b);
            }
        }

        #[test]
        fn t4() {
            for state in [State::Falling, State::Settled] {
                let a = Block {
                    state,
                    assigned_text: "abc",
                    input_text: "a".to_string(),
                    position: BoardPosition { x: 0, y: 0 },
                };
                let mut b = a.clone();
                assert!(!b.delete_char());
                assert_eq!(a, b);
            }
        }
    }
}
