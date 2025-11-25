use std::cmp::Ordering;

use derive_more::{Deref, DerefMut};
use getset::{CopyGetters, Getters};
use vector2d::Vector2D;

use super::Block;
use super::block::State as BlockState;
use super::block::bounding_box::MixAdd;

/// A position on the board. Origin is top left.
///
/// Positions are ordered by:
/// y descending (bottom of the board to top) then x ascending (left to right)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut, Hash)]
pub struct BoardPosition(pub Vector2D<u8>);
impl From<Vector2D<u8>> for BoardPosition {
    fn from(value: Vector2D<u8>) -> Self {
        Self(value)
    }
}
impl From<BoardPosition> for Vector2D<u8> {
    fn from(value: BoardPosition) -> Self {
        value.0
    }
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
impl<T> MixAdd<T> for BoardPosition
where
    Vector2D<u8>: MixAdd<T>,
{
    fn mix_add(self, other: T) -> Self {
        Self(self.0.mix_add(other))
    }
}

impl BoardPosition {
    #[inline]
    pub const fn new(x: u8, y: u8) -> Self {
        Self(Vector2D { x, y })
    }
    #[inline]
    pub const fn to_render_coords(
        self,
        cell_width: f64,
        cell_height: f64,
        anchor: AnchorCoordinates,
    ) -> RenderCoordinates {
        RenderCoordinates {
            x: self.0.x as f64 * cell_width + cell_width * anchor.x,
            y: self.0.y as f64 * cell_height + cell_height * anchor.y,
        }
    }

    #[inline]
    pub const fn center(self, cell_width: f64, cell_height: f64) -> RenderCoordinates {
        self.to_render_coords(
            cell_width,
            cell_height,
            AnchorCoordinates { x: 0.5, y: 0.5 },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderCoordinates {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, CopyGetters)]
pub struct AnchorCoordinates {
    #[getset(get_copy = "pub")]
    x: f64,
    #[getset(get_copy = "pub")]
    y: f64,
}
impl AnchorCoordinates {
    #[inline]
    pub const fn new(x: f64, y: f64) -> Option<Self> {
        if 0.0 <= x && x <= 1.0 && 0.0 <= y && y <= 1.0 {
            Some(Self { x, y })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    GameOver,
    BlocksSettled,
    Updated,
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
    pub fn new(width: u8, height: u8, starts_with_one: bool) -> Self {
        Self {
            blocks: if starts_with_one {
                vec![Block::random(width)]
            } else {
                vec![]
            },
            width,
            height,
        }
    }

    /// Clear completed rows and return the number of rows cleared.
    pub(super) fn clear_completed(&mut self) -> Vec<Block> {
        self.sort();
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

    pub(super) fn fall_tick(&mut self, include_interactable: bool) -> Option<Msg> {
        let mut newly_settled = false;
        let mut has_update = false;
        for i in 0..self.blocks.len() {
            let block = &mut self.blocks[i];
            if block.is_settled() || (!include_interactable && block.is_interactable()) {
                continue;
            }

            block.shift(Vector2D::new(0, 1));
            let block = &self.blocks[i];
            if self.blocks.iter().any(|b| block.intersect(&b)) {
                let block = &mut self.blocks[i];
                block.shift(Vector2D::new(0, -1));
                block.state = BlockState::Settled;
                newly_settled = true;
                if block.y() == 0 {
                    return Some(Msg::GameOver);
                }
            } else {
                has_update = true;
            }
        }
        if newly_settled {
            Some(Msg::BlocksSettled)
        } else {
            has_update.then_some(Msg::Updated)
        }
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
        self.blocks.push(Block::random(self.width));
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

    #[inline]
    pub(super) fn sort(&mut self) {
        self.blocks.sort_by(|a, b| match a.state.cmp(&b.state) {
            Ordering::Equal => a.position.cmp(&b.position),
            ord => ord,
        });
    }

    #[inline]
    pub(super) fn push_block(&mut self, block: Block) {
        self.blocks.push(block);
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
    pub(super) fn no_overlap(&self) -> bool {
        use iter_tools::Itertools;

        self.blocks
            .iter()
            .flat_map(|b| b.cells().iter().map(|c| c.position))
            .all_unique()
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
            let block = board.get_focused().unwrap();
            assert_eq!(block.x(), 4);
            assert_eq!(block.y(), 3);
            assert!(board.no_overlap());
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
            let block = board.get_focused().unwrap();
            assert_eq!(block.x(), 6);
            assert_eq!(block.y(), 3);
            assert!(board.no_overlap());
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
        fn drift(b: &mut Board) -> bool {
            b.fall_tick(true).is_some()
        }

        #[inline]
        fn fall(b: &mut Board) -> bool {
            b.fall_tick(false).is_some()
        }

        #[test]
        fn empty() {
            assert_unchanged(&empty_board(), drift);
        }

        #[test]
        fn all_settled() {
            assert_unchanged(&Board::populated(), drift);
        }

        #[test]
        fn single() {
            let mut board = Board::populated();
            let i = board.blocks.len();
            board.blocks.push(block(2, 20, "Bayanetta"));

            assert_unchanged(&board, fall);
            assert_eq!(board.fall_tick(true), Some(Msg::Updated));
            let block = &board.blocks[i];
            assert_eq!(block.x(), 2);
            assert_eq!(block.y(), 21);
            assert_eq!(board.get_focused_index(), Some(i));
            assert!(board.no_overlap());

            assert_eq!(board.fall_tick(true), Some(Msg::Updated));
            assert_eq!(board.fall_tick(true), Some(Msg::BlocksSettled));
            let block = &board.blocks[i];
            assert_eq!(block.x(), 2);
            assert_eq!(block.y(), 22);
            assert_eq!(board.get_focused_index(), None);
            assert!(board.no_overlap());

            assert_unchanged(&board, drift);
        }

        #[test]
        fn double() {
            let mut board = Board::populated();
            let i = board.blocks.len();
            board.blocks.push(block(2, 20, "Bayanetta"));
            board.blocks.push(block(7, 2, "Hornet"));

            assert_eq!(board.fall_tick(true), Some(Msg::Updated));
            assert!(board.no_overlap());
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 21 });
            assert_eq!(board.blocks[i + 1].position, BoardPosition { x: 7, y: 3 });
            assert_eq!(board.get_focused_index(), Some(i));

            assert_eq!(board.fall_tick(true), Some(Msg::Updated));
            assert!(board.no_overlap());
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 22 });
            assert_eq!(board.blocks[i + 1].position, BoardPosition { x: 7, y: 4 });
            assert_eq!(board.get_focused_index(), Some(i));

            assert_eq!(board.fall_tick(true), Some(Msg::BlocksSettled));
            assert!(board.no_overlap());
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
            assert_eq!(board.fall_tick(true), Some(Msg::Updated));
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 1 });
            assert_eq!(board.fall_tick(true), Some(Msg::BlocksSettled));
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 1 });

            let i = board.blocks.len();
            board.blocks.push(block(2, 0, "Bayanetta"));
            assert_eq!(board.fall_tick(true), Some(Msg::GameOver));
            assert_eq!(board.blocks[i].position, BoardPosition { x: 2, y: 0 });
        }

        #[test]
        fn settle_bottom() {
            let mut board = Board::new(4, 4, true);
            assert_eq!(board.fall_tick(true), Some(Msg::Updated));
            assert_eq!(board.fall_tick(true), Some(Msg::Updated));
            assert_eq!(board.fall_tick(true), Some(Msg::Updated));
            assert_eq!(board.fall_tick(true), Some(Msg::BlocksSettled));
            assert_unchanged(&board, drift);
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
