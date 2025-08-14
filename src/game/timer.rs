use getset::{CopyGetters, Getters};

#[derive(Debug, Clone, Copy, PartialEq, Eq, CopyGetters)]
pub(super) struct Msg {
    #[getset(get_copy = "pub(super)")]
    should_spawn: bool,
    #[getset(get_copy = "pub(super)")]
    should_fall: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Getters)]
pub(super) struct Timer {
    fall_interval: f64,
    spawn_interval: f64,
    spawn_timer: f64,
    fall_timer: f64,
    #[getset(get = "pub(super)")]
    last_delta: f64,
}

impl Timer {
    #[inline]
    pub(super) fn new(fall_interval: f64, spawn_interval: f64) -> Self {
        Self {
            fall_interval,
            spawn_interval,
            spawn_timer: spawn_interval,
            fall_timer: fall_interval,
            last_delta: 0.0,
        }
    }

    pub(super) fn tick(&mut self, delta_time: f64) -> Msg {
        self.last_delta = delta_time;
        self.fall_timer -= delta_time;
        self.spawn_timer -= delta_time;
        let ret = Msg {
            should_spawn: self.spawn_timer <= 0.0,
            should_fall: self.fall_timer <= 0.0,
        };
        if ret.should_spawn {
            self.spawn_timer += self.spawn_interval;
        }
        if ret.should_fall {
            self.fall_timer += self.fall_interval;
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn t0() {
        let mut timer = Timer::new(1.0, 2.0);
        assert_eq!(
            timer.tick(0.7),
            Msg {
                should_spawn: false,
                should_fall: false
            }
        );
        assert_eq!(timer.last_delta(), &0.7);
        assert_eq!(
            timer.tick(0.7),
            Msg {
                should_spawn: false,
                should_fall: true
            }
        );
        assert_eq!(timer.last_delta(), &0.7);
        assert_eq!(
            timer.tick(0.8),
            Msg {
                should_spawn: true,
                should_fall: true
            }
        );
        assert_eq!(timer.last_delta(), &0.8);
        assert_eq!(
            timer.tick(0.2),
            Msg {
                should_spawn: false,
                should_fall: false
            }
        );
        assert_eq!(timer.last_delta(), &0.2);
    }
}
