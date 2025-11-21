use std::cmp::Ordering;

use getset::{CopyGetters, Getters};

use super::Block;
use super::block::State as BlockState;

/// A position on the board. Origin is top left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoardPosition {
    pub x: u8,
    pub y: u8,
}
impl Ord for BoardPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.y.cmp(&self.y) {
            Ordering::Equal => self.x.cmp(&other.x),
            ord => ord,
        }
    }
}
impl PartialOrd for BoardPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl BoardPosition {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    GameOver,
    BlocksSettled,
}

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters, Getters)]
pub struct Board {
    #[getset(get = "pub")]
    blocks: Vec<Block>,
    #[getset(get_copy = "pub")]
    width: u8,
    #[getset(get_copy = "pub")]
    height: u8,
}

impl Board {
    #[inline]
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            blocks: Vec::new(),
            width,
            height,
        }
    }

    /// Find the max y of the block at `target_index` to prevent overlapping with other blocks
    /// below it.
    pub(super) fn find_max_y(&self, target_index: usize) -> u8 {
        let target_block = &self.blocks[target_index];
        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(i, block)| {
                (i != target_index
                    && block.is_settled()
                    && block.position.y > target_block.position.y
                    && block.intersect_x(target_block))
                .then_some(block.position.y)
            })
            .min()
            .unwrap_or(self.height)
            - 1
    }

    /// Clear completed rows and return the number of rows cleared.
    pub(super) fn clear_completed(&mut self) -> Vec<Block> {
        self.blocks.sort_by(|a, b| match a.state.cmp(&b.state) {
            Ordering::Equal => a.position.cmp(&b.position),
            ord => ord,
        });
        let mut removals = Vec::new();
        for chunk in self
            .blocks
            .chunk_by_mut(|a, b| a.position.y == b.position.y)
        {
            let Some(first) = chunk.first() else {
                continue;
            };
            if !first.is_settled() {
                break;
            }
            if first.position.x != 0 {
                continue;
            }

            let mut filled = true;
            for window in chunk.windows(2) {
                let [a, b] = window else {
                    unreachable!();
                };

                if a.position.x + a.width() != b.position.x {
                    filled = false;
                    break;
                }
            }

            if filled
                && let Some(b) = chunk.last()
                && b.position.x + b.width() == self.width
            {
                removals.push(b.position.y);
            }
        }

        if let Some(&max_y) = removals.first() {
            self.blocks
                .extract_if(.., |b| {
                    if removals.contains(&b.position.y) {
                        return true;
                    }
                    if b.is_settled() && b.position.y < max_y {
                        b.state = BlockState::Falling;
                    }
                    false
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub(super) fn fall_tick(&mut self) -> Option<Msg> {
        let mut newly_settled = false;
        for i in 0..self.blocks.len() {
            if self.blocks[i].is_settled() {
                continue;
            }
            let max_y = self.find_max_y(i);
            if max_y == 0 {
                return Some(Msg::GameOver);
            }
            let block = &mut self.blocks[i];
            block.position.y += 1;
            if block.position.y == max_y {
                block.state = BlockState::Settled;
                newly_settled = true;
            }
        }
        newly_settled.then_some(Msg::BlocksSettled)
    }

    #[inline]
    pub fn get_focused_index(&self) -> Option<usize> {
        self.blocks.iter().position(|b| b.is_interactable())
    }

    #[inline]
    pub fn get_focused(&self) -> Option<&Block> {
        self.blocks.iter().find(|b| b.is_interactable())
    }

    #[inline]
    pub(super) fn get_focused_mut(&mut self) -> Option<&mut Block> {
        self.blocks.iter_mut().find(|b| b.is_interactable())
    }

    #[inline]
    pub(super) fn spawn_block(&mut self) {
        self.blocks.push(Block::new(self.width));
    }

    #[inline]
    pub(super) fn focus_next(&mut self) -> bool {
        if let Some(focus) = self.get_focused_mut() {
            focus.state = BlockState::Falling;
            true
        } else {
            false
        }
    }

    pub(super) fn left(&mut self) -> bool {
        let Some(focus_index) = self.get_focused_index() else {
            return false;
        };
        let focus = &self.blocks[focus_index];
        if !focus.is_movable() {
            return false;
        }
        let x = focus.position.x;
        if x == 0 {
            return false;
        }
        let y = focus.position.y;
        for block in self.blocks.iter().take_while(|b| b.is_settled()) {
            if block.position.y == y && block.position.x + block.width() == x {
                return false;
            }
        }
        self.blocks[focus_index].position.x -= 1;
        true
    }

    pub(super) fn right(&mut self) -> bool {
        let Some(focus_index) = self.get_focused_index() else {
            return false;
        };
        let focus = &self.blocks[focus_index];
        if !focus.is_movable() {
            return false;
        };
        let x = focus.position.x + focus.width();
        if x == self.width {
            return false;
        }
        let y = focus.position.y;
        for block in self.blocks.iter().take_while(|b| b.is_settled()) {
            if block.position.y == y && block.position.x == x {
                return false;
            }
        }
        self.blocks[focus_index].position.x += 1;
        true
    }
}

#[cfg(test)]
impl Board {
    #[inline]
    pub(super) fn populated() -> Board {
        Board {
            blocks: vec![
                Block::new_settled("Taylor", 10, 31),
                Block::new_settled("hello", 0, 31),
                Block::new_settled("world", 5, 31),
                Block::new_settled("Supercali", 3, 30),
                Block::new_settled("Rustaceanvim", 2, 29),
                Block::new_settled("LazyVim", 0, 28),
                Block::new_settled("folke", 7, 28),
                Block::new_settled("four", 12, 28),
                Block::new_settled("Stranger", 1, 27),
                Block::new_settled("Contessa", 3, 26),
                Block::new_settled("im", 7, 25),
                Block::new_settled("outtacotta", 4, 24),
                Block::new_settled("ideas", 5, 23),
            ],
            width: 16,
            height: 32,
        }
    }

    #[inline]
    pub(super) fn push_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[inline]
    fn assert_unchanged(a: &Board, f: impl FnOnce(&mut Board) -> bool) {
        let mut b = a.clone();
        assert!(!f(&mut b));
        assert_eq!(a, &b);
    }

    #[inline]
    fn block(x: u8, y: u8, text: &'static str) -> Block {
        Block::new_interactable(text, x, y)
    }

    #[inline]
    fn empty_board() -> Board {
        Board {
            blocks: vec![],
            width: 16,
            height: 24,
        }
    }

    mod left {
        use super::*;

        #[inline]
        fn f(b: &mut Board) -> bool {
            b.left()
        }

        #[test]
        fn t0() {
            assert_unchanged(
                &Board {
                    blocks: vec![block(0, 3, "abc")],
                    width: 24,
                    height: 32,
                },
                f,
            );
        }

        #[test]
        fn t1() {
            let mut board = Board {
                blocks: vec![block(5, 3, "abc")],
                width: 24,
                height: 32,
            };
            assert!(!board.left());
            let block = board.get_focused_mut().unwrap();
            block.add_char('a');
            block.add_char('b');
            block.add_char('c');
            assert!(board.left());
            assert!(
                board
                    .get_focused()
                    .is_some_and(|b| b.position == BoardPosition { x: 4, y: 3 })
            )
        }

        #[test]
        fn t2() {
            let mut a = Board::populated();
            a.blocks.push(block(10, 23, "abc"));
            assert_unchanged(&a, f);
        }
    }

    mod right {
        use super::*;

        #[inline]
        fn f(b: &mut Board) -> bool {
            b.right()
        }

        #[test]
        fn t0() {
            assert_unchanged(
                &Board {
                    blocks: vec![block(21, 3, "abc")],
                    width: 24,
                    height: 32,
                },
                f,
            );
        }

        #[test]
        fn t1() {
            let mut board = Board {
                blocks: vec![block(5, 3, "abc")],
                width: 24,
                height: 32,
            };
            assert!(!board.right());
            let block = board.get_focused_mut().unwrap();
            block.add_char('a');
            block.add_char('b');
            block.add_char('c');
            assert!(board.right());
            assert!(
                board
                    .get_focused()
                    .is_some_and(|b| b.position == BoardPosition { x: 6, y: 3 })
            )
        }

        #[test]
        fn t2() {
            let mut a = Board::populated();
            a.blocks.push(block(2, 23, "abc"));
            assert_unchanged(&a, f);
        }
    }

    mod focus {
        use super::*;

        #[inline]
        fn f(b: &mut Board) -> bool {
            b.focus_next()
        }

        #[test]
        fn empty() {
            let mut board = empty_board();
            assert!(board.get_focused().is_none());
            assert!(board.get_focused_mut().is_none());
            assert!(board.get_focused_index().is_none());
            assert_unchanged(&board, f);
        }

        #[test]
        fn t0() {
            let mut board = Board::populated();
            let i = board.blocks.len();
            let a = block(5, 10, "abc");
            let b = block(5, 5, "abc");
            board.blocks.push(a.clone());
            board.blocks.push(b.clone());
            assert!(board.get_focused().is_some_and(|x| &a == x));
            assert!(board.get_focused_mut().is_some_and(|x| &a == x));
            assert!(board.get_focused_index().is_some_and(|j| i == j));
            assert!(board.focus_next());
            assert!(board.get_focused().is_some_and(|x| &b == x));
            assert!(board.get_focused_mut().is_some_and(|x| &b == x));
            assert!(board.get_focused_index().is_some_and(|j| i + 1 == j));
            assert!(board.focus_next());
            assert!(board.get_focused().is_none());
            assert!(board.get_focused_mut().is_none());
            assert!(board.get_focused_index().is_none());
        }
    }

    mod fall {
        use super::*;

        #[inline]
        fn f(b: &mut Board) -> bool {
            b.fall_tick().is_some()
        }

        #[test]
        fn empty() {
            assert_unchanged(&empty_board(), f);
        }

        #[test]
        fn all_settled() {
            assert_unchanged(&Board::populated(), f);
        }

        #[test]
        fn single() {
            let mut board = Board::populated();
            let i = board.blocks.len();
            board.blocks.push(block(2, 20, "Bayanetta"));

            assert!(board.fall_tick().is_none());
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 21 });
            assert_eq!(board.get_focused_index(), Some(i));

            assert_eq!(board.fall_tick(), Some(Msg::BlocksSettled));
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 22 });
            assert_eq!(board.get_focused_index(), None);

            assert_unchanged(&board, f);
        }

        #[test]
        fn double() {
            let mut board = Board::populated();
            let i = board.blocks.len();
            board.blocks.push(block(2, 20, "Bayanetta"));
            board.blocks.push(block(7, 2, "Hornet"));

            assert!(board.fall_tick().is_none());
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 21 });
            assert_eq!(board.blocks[i + 1].position, BoardPosition { x: 7, y: 3 });
            assert_eq!(board.get_focused_index(), Some(i));

            assert_eq!(board.fall_tick(), Some(Msg::BlocksSettled));
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 22 });
            assert_eq!(board.blocks[i + 1].position, BoardPosition { x: 7, y: 4 });
            assert_eq!(board.get_focused_index(), Some(i + 1));

            assert!(board.fall_tick().is_none());
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 22 });
            assert_eq!(board.blocks[i + 1].position, BoardPosition { x: 7, y: 5 });
            assert_eq!(board.get_focused_index(), Some(i + 1));
        }

        #[test]
        fn full() {
            let mut board = Board::populated();
            board.blocks.extend(
                (2..23)
                    .rev()
                    .map(|y| Block::new_settled("Stupendium", 0, y)),
            );
            let i = board.blocks.len();
            board.blocks.push(block(2, 0, "Bayanetta"));
            assert_eq!(board.fall_tick(), Some(Msg::BlocksSettled));
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 1 });

            let i = board.blocks.len();
            board.blocks.push(block(2, 0, "Bayanetta"));
            assert_eq!(board.fall_tick(), Some(Msg::GameOver));
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 0 });
        }
    }

    mod clear {
        use super::*;

        #[test]
        fn empty() {
            assert_unchanged(&empty_board(), |b| !b.clear_completed().is_empty());
        }

        #[test]
        fn t0() {
            let mut board = Board::populated();
            assert_eq!(
                board.clear_completed(),
                &[
                    Block::new_settled("hello", 0, 31),
                    Block::new_settled("world", 5, 31),
                    Block::new_settled("Taylor", 10, 31),
                    Block::new_settled("LazyVim", 0, 28),
                    Block::new_settled("folke", 7, 28),
                    Block::new_settled("four", 12, 28),
                ]
            );
            assert_eq!(
                board.blocks,
                vec![
                    Block::new_falling("Supercali", 3, 30),
                    Block::new_falling("Rustaceanvim", 2, 29),
                    Block::new_falling("Stranger", 1, 27),
                    Block::new_falling("Contessa", 3, 26),
                    Block::new_falling("im", 7, 25),
                    Block::new_falling("outtacotta", 4, 24),
                    Block::new_falling("ideas", 5, 23),
                ]
            );
            assert_unchanged(&empty_board(), |b| !b.clear_completed().is_empty());
        }
    }

    #[test]
    fn find_max_y() {
        let mut board = Board::populated();
        let i = board.blocks.len();
        board.blocks.push(block(2, 20, "Bayanetta"));
        assert_eq!(board.find_max_y(i), 22);

        // only settled blocks are considered for the purpose of finding max y
        board.blocks.push(block(7, 2, "Hornet"));
        assert_eq!(board.find_max_y(i + 1), 22);
    }
}
