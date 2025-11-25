pub mod block;
pub mod board;
mod english;
pub mod settings;
mod timer;

use std::collections::BTreeSet;

use block::Block;
use board::Board;
use getset::{CopyGetters, Getters, WithSetters};
use settings::Settings;
use timer::Timer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Splash,
    Playing,
    GameOver,
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Tick(f64),
    Type(char),
    Delete,
    Next,
    Left,
    Right,
    NewGame,
}

#[derive(Debug, Clone, PartialEq, CopyGetters, Getters, WithSetters)]
pub struct Game {
    #[getset(get = "pub", set_with)]
    settings: Settings,
    #[getset(get_copy = "pub")]
    state: State,
    #[getset(get = "pub")]
    board: Board,
    timer: Timer,
    #[getset(get_copy = "pub")]
    score: usize,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Settings::default())
    }
}

impl Game {
    pub fn splash() -> Self {
        let mut board = Board::new(12, 16, false);
        use block::State as S;
        board.push_block(
            Block::new_line("Typetris", S::Interactable, 2, 0)
                .expect("static ascii text should be valid."),
        );
        board.push_block(
            Block::new_line("It's", S::Interactable, 0, 4)
                .expect("static ascii text should be valid."),
        );
        board.push_block(
            Block::new_line("like", S::Interactable, 3, 5)
                .expect("static ascii text should be valid."),
        );
        board.push_block(
            Block::new_line("Tetris", S::Interactable, 6, 6)
                .expect("static ascii text should be valid."),
        );
        board.push_block(
            Block::new_line("but", S::Interactable, 8, 8)
                .expect("static ascii text should be valid."),
        );
        board.push_block(
            Block::new_line("make", S::Interactable, 4, 9)
                .expect("static ascii text should be valid."),
        );
        board.push_block(
            Block::new_line("it", S::Interactable, 2, 10)
                .expect("static ascii text should be valid."),
        );
        board.push_block(
            Block::new_line("a", S::Interactable, 4, 11)
                .expect("static ascii text should be valid."),
        );
        let mut block = Block::new_line("typing", S::Interactable, 5, 12)
            .expect("static ascii text should be valid.");
        block.add_char('t');
        block.add_char('y');
        block.add_char('i');
        block.add_char('n');
        board.push_block(block);
        board.push_block(
            Block::new_line("game", S::Settled, 4, 15).expect("static ascii text should be valid."),
        );
        board.sort();
        let settings = Settings::default().with_width(12).with_height(16);
        Self {
            board,
            timer: Timer::new(
                settings.fall_interval,
                settings.spawn_interval,
                settings.drift_interval,
            ),
            score: 0,
            state: State::Splash,
            settings,
        }
    }

    #[inline]
    pub fn new(settings: Settings) -> Self {
        if settings.starts_with_splash {
            Self::splash().with_settings(settings.with_starts_with_splash(false))
        } else {
            Self {
                board: Board::new(settings.width, settings.height, settings.starts_with_one),
                timer: Timer::new(
                    settings.fall_interval,
                    settings.spawn_interval,
                    settings.drift_interval,
                ),
                score: 0,
                state: State::Playing,
                settings,
            }
        }
    }

    /// Handle the given event and return a boolean indicating whether state has changed.
    #[inline]
    pub fn handle_event(&mut self, event: Event) -> bool {
        match (self.is_playing(), event) {
            (_, Event::NewGame) => {
                self.new_game();
                true
            }
            (true, Event::Tick(delta_time)) => self.tick(delta_time),
            (true, Event::Type(ch)) => self.add_char(ch),
            (true, Event::Delete) => self.delete_char(),
            (true, Event::Next) => self.focus_next(),
            (true, Event::Left) => self.left(),
            (true, Event::Right) => self.right(),
            (false, _) => false,
        }
    }

    fn tick(&mut self, delta_time: f64) -> bool {
        let timer_msg = self.timer.tick(delta_time);

        let mut ret = false;
        if timer_msg.should_fall() {
            ret = true;
            use board::Msg as M;
            match self.board.fall_tick(timer_msg.should_drift()) {
                Some(M::GameOver) => self.state = State::GameOver,
                Some(M::BlocksSettled) => {
                    let cleared = self.board.clear_completed();
                    self.score += cleared.iter().map(|b| b.y()).collect::<BTreeSet<_>>().len()
                }
                Some(M::Updated) => (),
                None => ret = false,
            }
        }
        if timer_msg.should_spawn() {
            self.board.spawn_block();
            ret = true;
            if self.board.find_max_y(self.board.blocks().len() - 1) == 0 {
                self.state = State::GameOver;
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

    #[inline]
    pub fn is_game_over(&self) -> bool {
        self.state == State::GameOver
    }

    #[inline]
    pub fn is_playing(&self) -> bool {
        self.state == State::Playing
    }

    #[inline]
    pub fn is_splash(&self) -> bool {
        self.state == State::Splash
    }

    #[inline]
    fn new_game(&mut self) {
        *self = Self::new(self.settings);
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
                    .is_some_and(|b| b.check_input_text(expected_text))
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
            game.handle_event(Event::Tick(1.0));
            if game.is_game_over() {
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
            let text = game
                .board
                .get_focused()
                .unwrap()
                .cells()
                .iter()
                .map(|c| char::from(c.assigned_char))
                .collect::<Vec<_>>();
            for ch in text {
                game.handle_event(E::Type(ch));
            }
            for _ in 0..10 {
                game.handle_event(E::Left);
            }
            game.handle_event(E::Next);
            game.handle_event(E::Tick(1.0));
            game.handle_event(E::Tick(1.0));
        }

        assert_eq!(game.state, State::GameOver);
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

        assert!(!game.handle_event(E::Tick(0.25)));
        repeat_event(&mut game, E::Tick(0.2), 4);
        assert!(dbg!(game.board.get_focused()).is_some_and(|b| b.y() == 1));

        assert!(game.handle_event(E::Next));
        repeat_event(&mut game, E::Tick(0.2), 14);
        assert!(dbg!(game.board.blocks().first()).is_some_and(|b| b.y() == 15));
        assert!(dbg!(game.board.get_focused()).is_some_and(|b| b.y() == 1));
        assert!(game.handle_event(E::Tick(0.2)));
        assert!(dbg!(game.board.get_focused()).is_some_and(|b| b.y() == 2));
        assert!(dbg!(game.board.blocks().first()).is_some_and(|b| b.is_settled() && b.y() == 15));
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
        assert!(!game.handle_event(Event::Tick(0.25)));
        assert!(game.handle_event(Event::Left));
        assert!(!game.handle_event(Event::Tick(0.2)));
        assert!(game.handle_event(Event::Left));
        assert!(game.handle_event(Event::Tick(0.2)));
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
        assert_eq!(game.state, State::GameOver);
    }

    #[test]
    fn splash() {
        let settings = Settings::default()
            .with_fall_interval(1.0)
            .with_drift_interval(1)
            .with_starts_with_splash(true);
        let mut game = Game::new(settings);
        assert_eq!(game.state, State::Splash);
        assert_eq!(game.board, Game::splash().board);
        assert_unchanged(&game, |g| !g.handle_event(Event::Tick(1.1)));
        assert_unchanged(&game, |g| !g.handle_event(Event::Left));
        assert_unchanged(&game, |g| !g.handle_event(Event::Right));
        assert_unchanged(&game, |g| !g.handle_event(Event::Type('a')));
        assert_unchanged(&game, |g| !g.handle_event(Event::Delete));
        assert!(game.handle_event(Event::NewGame));
        assert_eq!(game.board.blocks().len(), 1);
        assert_eq!(game.board.width(), settings.width);
        assert_eq!(game.board.height(), settings.height);
        assert!(game.handle_event(Event::Tick(1.1)));
        assert!(game.handle_event(Event::Type('a')));
    }
}
