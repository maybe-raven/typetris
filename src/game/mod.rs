pub mod block;
pub mod board;
pub mod settings;
mod timer;

use std::collections::BTreeSet;

use block::Block;
use board::Board;
use getset::{CopyGetters, Getters};
use settings::Settings;
use timer::Timer;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Tick(f64),
    Type(char),
    Delete,
    Next,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, CopyGetters, Getters)]
pub struct Game {
    #[getset(get = "pub")]
    board: Board,
    timer: Timer,
    #[getset(get_copy = "pub")]
    score: usize,
    #[getset(get_copy = "pub")]
    game_over: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Settings::default())
    }
}

impl Game {
    #[inline]
    pub fn new(settings: Settings) -> Self {
        Self {
            board: Board::new(settings.width, settings.height, settings.starts_with_one),
            timer: Timer::new(
                settings.fall_interval,
                settings.spawn_interval,
                settings.drift_interval,
            ),
            score: 0,
            game_over: false,
        }
    }

    /// Handle the given event and return a boolean indicating whether state has changed.
    #[inline]
    pub fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Tick(delta_time) => self.tick(delta_time),
            Event::Type(ch) => self.add_char(ch),
            Event::Delete => self.delete_char(),
            Event::Next => self.focus_next(),
            Event::Left => self.left(),
            Event::Right => self.right(),
        }
    }

    fn tick(&mut self, delta_time: f64) -> bool {
        if self.game_over {
            return false;
        }

        let timer_msg = self.timer.tick(delta_time);

        let mut ret = false;
        if timer_msg.should_fall() {
            use board::Msg as M;
            match self.board.fall_tick(timer_msg.should_drift()) {
                Some(M::GameOver) => self.game_over = true,
                Some(M::BlocksSettled) => {
                    let cleared = self.board.clear_completed();
                    self.score += cleared
                        .iter()
                        .map(|b| b.position.y)
                        .collect::<BTreeSet<_>>()
                        .len()
                }
                None => (),
            }
            ret = true;
        }
        if timer_msg.should_spawn() {
            self.board.spawn_block();
            ret = true;
            if self.board.find_max_y(self.board.blocks().len() - 1) == 0 {
                self.game_over = true;
            }
        }

        ret
    }

    #[inline]
    fn add_char(&mut self, ch: char) -> bool {
        if let Some(focus) = self.board.get_focused_mut() {
            focus.add_char(ch)
        } else {
            false
        }
    }

    #[inline]
    fn delete_char(&mut self) -> bool {
        if let Some(focus) = self.board.get_focused_mut() {
            focus.delete_char()
        } else {
            false
        }
    }

    #[inline]
    fn focus_next(&mut self) -> bool {
        self.board.focus_next()
    }

    #[inline]
    fn left(&mut self) -> bool {
        self.board.left()
    }

    #[inline]
    fn right(&mut self) -> bool {
        self.board.right()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[inline]
    fn assert_unchanged(a: &Game, f: impl FnOnce(&mut Game) -> bool) {
        let mut b = a.clone();
        assert!(f(&mut b));
        assert_eq!(a, &b);
    }

    #[inline]
    fn fall_until_settled(game: &mut Game) {
        while !matches!(game.board.fall_tick(true), Some(board::Msg::BlocksSettled)) {}
    }

    #[inline]
    fn repeat_event(game: &mut Game, event: Event, n: usize) {
        for _ in 0..n {
            game.handle_event(event);
        }
    }

    mod typing {
        use super::*;

        #[inline]
        fn handle_and_assert(g: &mut Game, event: Event, expected_text: &str) {
            assert!(g.handle_event(event));
            assert!(
                g.board
                    .get_focused()
                    .is_some_and(|b| b.input_text() == expected_text)
            );
        }

        #[test]
        fn settled_unchanged() {
            let mut game = Game::default();
            fall_until_settled(&mut game);
            assert_unchanged(&game, |g| !g.add_char('a'));
            assert_unchanged(&game, |g| !g.delete_char());
        }

        #[test]
        fn empty_unchanged() {
            let game = Game::new(Settings::default().with_starts_with_one(false));
            assert_unchanged(&game, |g| !g.add_char('a'));
            assert_unchanged(&game, |g| !g.delete_char());
        }

        #[test]
        fn complex() {
            let mut game = Game::new(Settings::default().with_starts_with_one(false));
            game.board.push_block(Block::with_text_x("one", 1));

            // one block at the top
            handle_and_assert(&mut game, Event::Type('a'), "a");
            handle_and_assert(&mut game, Event::Type('b'), "ab");
            handle_and_assert(&mut game, Event::Delete, "a");

            game.board.fall_tick(true);
            game.board.fall_tick(true);
            game.board.fall_tick(true);
            game.board.push_block(Block::with_text_x("two", 4));

            // one block in the middle (focus), one block at the top
            handle_and_assert(&mut game, Event::Type('m'), "am");

            fall_until_settled(&mut game);
            // one block settled at the bottom, one block in the middle (focus)
            handle_and_assert(&mut game, Event::Type('r'), "r");
            handle_and_assert(&mut game, Event::Type('m'), "rm");
            game.board.push_block(Block::with_text_x("three", 0));
            // one block settled at the bottom, one block in the middle (focus),
            // one block at the top
            handle_and_assert(&mut game, Event::Delete, "r");
            handle_and_assert(&mut game, Event::Delete, "");
            handle_and_assert(&mut game, Event::Type('d'), "d");

            game.focus_next();
            handle_and_assert(&mut game, Event::Type('g'), "g");
            handle_and_assert(&mut game, Event::Type('g'), "gg");

            game.focus_next();
            assert_unchanged(&game, |g| !g.handle_event(Event::Type('x')));
        }
    }

    #[test]
    fn t0() {
        let settings = Settings::default()
            .with_spawn_interval(6.0)
            .with_fall_interval(1.0)
            .with_drift_interval(1);
        let mut game = Game::new(settings);
        game.board = Board::populated();
        game.board.push_block(Block::with_text_x("rusty", 2));

        assert!(game.handle_event(Event::Tick(1.1)));
        assert!(!game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Type('r')));
        assert!(game.handle_event(Event::Type('u')));
        assert!(game.handle_event(Event::Type('s')));
        assert!(game.handle_event(Event::Type('t')));
        assert!(game.handle_event(Event::Type('y')));
        assert!(!game.handle_event(Event::Type('p')));

        assert!(!game.handle_event(Event::Tick(0.1)));
        assert!(game.handle_event(Event::Tick(1.0)));
        assert!(game.handle_event(Event::Tick(1.0)));
        game.board.push_block(Block::with_text_x("kitten", 8));

        assert!(game.handle_event(Event::Left));
        assert!(game.handle_event(Event::Left));
        assert!(!game.handle_event(Event::Left));

        assert!(game.handle_event(Event::Tick(1.0)));
        assert!(game.handle_event(Event::Tick(1.0)));
        assert!(game.handle_event(Event::Tick(1.0)));
        assert!(game.handle_event(Event::Next));
        assert!(game.handle_event(Event::Type('k')));
        assert!(game.handle_event(Event::Type('i')));
        assert!(game.handle_event(Event::Type('t')));
        assert!(game.handle_event(Event::Type('t')));
        assert!(game.handle_event(Event::Type('e')));
        assert!(game.handle_event(Event::Type('n')));
        assert!(game.handle_event(Event::Tick(1.0)));
        assert!(game.handle_event(Event::Tick(1.0)));
        assert!(game.handle_event(Event::Tick(1.0)));
        assert!(game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Right));

        for _ in 0..20 {
            assert!(game.handle_event(Event::Tick(1.0)));
        }

        assert_eq!(game.score, 3);

        for _ in 0..500 {
            assert!(game.handle_event(Event::Tick(1.0)));
            if game.game_over {
                return;
            }
        }
        panic!("the game should end after running without playing for a while");
    }

    #[test]
    fn t1() {
        // The longest word in the current dictionary is 8 letters long, so a 10-wide board means
        // one block won't fill a row by itself.
        let settings = Settings::default()
            .with_starts_with_one(true)
            .with_width(10)
            .with_height(4)
            .with_spawn_interval(2.0)
            .with_fall_interval(0.25)
            .with_drift_interval(4);
        let mut game = Game::new(settings);
        use Event as E;

        for _ in 0..4 {
            let text = *game.board.get_focused().unwrap().assigned_text();
            for ch in text.chars() {
                game.handle_event(E::Type(ch));
            }
            for _ in 0..10 {
                game.handle_event(E::Left);
            }
            game.handle_event(E::Next);
            game.handle_event(E::Tick(1.0));
            game.handle_event(E::Tick(1.0));
        }

        assert!(game.game_over);
    }

    #[test]
    fn t2() {
        let settings = Settings::default()
            .with_starts_with_one(false)
            .with_width(4)
            .with_height(4)
            .with_spawn_interval(4.0)
            .with_fall_interval(1.0)
            .with_drift_interval(4);
        let mut game = Game::new(settings);
        use Event as E;

        game.board.push_block(Block::with_text_x("an", 0));
        assert!(game.handle_event(E::Next));
        assert!(game.handle_event(E::Tick(1.1)));
        assert!(game.handle_event(E::Tick(1.0)));
        game.board.push_block(Block::with_text_x("ny", 2));
        assert!(game.handle_event(E::Next));
        assert!(game.handle_event(E::Tick(1.0)));
        assert!(game.handle_event(E::Tick(1.0)));
        // new block spawns here after 4.1 time unit.
        assert!(game.handle_event(E::Tick(1.0)));
        assert!(game.handle_event(E::Tick(1.0)));
        // blocks settle and clear
        assert_eq!(game.board.blocks().len(), 1);
        assert_eq!(game.score, 1);
        assert!(game.board.get_focused().is_some());
    }

    #[test]
    fn free_fall() {
        let settings = Settings::default()
            .with_starts_with_one(true)
            .with_spawn_interval(2.0)
            .with_fall_interval(0.2)
            .with_drift_interval(5);
        let mut game = Game::new(settings);
        use Event as E;

        game.handle_event(E::Tick(0.25));
        repeat_event(&mut game, E::Tick(0.2), 4);
        assert!(dbg!(game.board.get_focused()).is_some_and(|b| b.position.y == 1));

        assert!(game.handle_event(E::Next));
        repeat_event(&mut game, E::Tick(0.2), 14);
        assert!(dbg!(game.board.blocks().first()).is_some_and(|b| b.position.y == 15));
        assert!(dbg!(game.board.get_focused()).is_some_and(|b| b.position.y == 1));
        game.handle_event(E::Tick(0.2));
        assert!(dbg!(game.board.get_focused()).is_some_and(|b| b.position.y == 2));
        assert!(
            dbg!(game.board.blocks().first()).is_some_and(|b| b.is_settled() && b.position.y == 15)
        );
    }

    #[test]
    fn skim_left() {
        let settings = Settings::default()
            .with_starts_with_one(false)
            .with_width(8)
            .with_height(8)
            .with_fall_interval(0.2)
            .with_drift_interval(1);
        let mut game = Game::new(settings);
        for y in 4..7 {
            game.board.push_block(Block::new_settled("why", 0, y));
        }
        game.board.push_block(Block::new_interactable("me", 3, 3));
        assert!(game.handle_event(Event::Type('m')));
        assert!(game.handle_event(Event::Type('e')));
        assert!(game.handle_event(Event::Left));
        assert!(game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Tick(0.25)));
        assert!(!game.handle_event(Event::Left));
        assert!(game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Left));
    }

    #[test]
    fn skim_right() {
        let settings = Settings::default()
            .with_starts_with_one(false)
            .with_width(8)
            .with_height(8)
            .with_fall_interval(0.2)
            .with_drift_interval(1);
        let mut game = Game::new(settings);
        for y in 4..7 {
            game.board.push_block(Block::new_settled("why", 5, y));
        }
        game.board.push_block(Block::new_interactable("me", 3, 3));
        assert!(game.handle_event(Event::Type('m')));
        assert!(game.handle_event(Event::Type('e')));
        assert!(game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Left));
        assert!(game.handle_event(Event::Tick(0.25)));
        assert!(!game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Left));
        assert!(game.handle_event(Event::Right));
    }

    #[test]
    fn skim_settle() {
        let settings = Settings::default()
            .with_starts_with_one(false)
            .with_width(8)
            .with_height(8)
            .with_fall_interval(0.2)
            .with_drift_interval(3);
        let mut game = Game::new(settings);
        for y in 4..7 {
            game.board.push_block(Block::new_settled("Cotton", 0, y));
        }
        game.board.push_block(Block::new_interactable("me", 6, 3));
        assert!(game.handle_event(Event::Type('m')));
        assert!(game.handle_event(Event::Type('e')));
        assert!(game.handle_event(Event::Left));
        game.handle_event(Event::Tick(0.25));
        assert!(game.handle_event(Event::Left));
        game.handle_event(Event::Tick(0.2));
        assert!(game.handle_event(Event::Left));
        game.handle_event(Event::Tick(0.2));
        assert!(game.board.get_focused().is_none());
    }

    #[test]
    fn full() {
        let settings = Settings::default()
            .with_starts_with_one(false)
            .with_width(8)
            .with_height(8)
            .with_fall_interval(1.0)
            .with_drift_interval(1);
        let mut game = Game::new(settings);
        for y in 1..7 {
            game.board.push_block(Block::new_settled("Rennoir", 0, y));
        }
        game.board
            .push_block(Block::new_interactable("Aline", 0, 0));
        assert!(!game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Type('A')));
        assert!(game.handle_event(Event::Type('l')));
        assert!(game.handle_event(Event::Type('i')));
        assert!(game.handle_event(Event::Type('c')));
        assert!(game.handle_event(Event::Delete));
        assert!(game.handle_event(Event::Type('n')));
        assert!(game.handle_event(Event::Type('e')));
        assert!(game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Right));
        assert!(!game.handle_event(Event::Right));
        assert!(game.handle_event(Event::Tick(1.1)));
        assert!(game.game_over);
    }
}
