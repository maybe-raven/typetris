use getset::{CopyGetters, Getters};

#[derive(Debug, Clone, Copy, PartialEq, Eq, CopyGetters)]
pub(super) struct Msg {
    #[getset(get_copy = "pub(super)")]
    should_spawn: bool,
    #[getset(get_copy = "pub(super)")]
    should_fall: bool,
    #[getset(get_copy = "pub(super)")]
    should_drift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Getters)]
pub(super) struct Timer {
    drift_interval: u8,
    fall_count: u8,
    fall_interval: f64,
    spawn_interval: f64,
    spawn_timer: f64,
    fall_timer: f64,
    #[getset(get = "pub(super)")]
    last_delta: f64,
}

impl Timer {
    #[inline]
    pub(super) fn new(fall_interval: f64, spawn_interval: f64, drift_interval: u8) -> Self {
        Self {
            drift_interval,
            fall_count: drift_interval,
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
        let mut ret = Msg {
            should_spawn: self.spawn_timer <= 0.0,
            should_fall: self.fall_timer <= 0.0,
            should_drift: false,
        };
        if ret.should_spawn {
            self.spawn_timer += self.spawn_interval;
        }
        if ret.should_fall {
            self.fall_timer += self.fall_interval;
            self.fall_count -= 1;
        }
        if self.fall_count == 0 {
            ret.should_drift = true;
            self.fall_count = self.drift_interval;
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn t0() {
        let mut timer = Timer::new(1.0, 2.0, 2);
        assert_eq!(
            timer.tick(0.7),
            Msg {
                should_spawn: false,
                should_fall: false,
                should_drift: false
            }
        );
        assert_eq!(timer.last_delta(), &0.7);
        assert_eq!(
            timer.tick(0.7),
            Msg {
                should_spawn: false,
                should_fall: true,
                should_drift: false
            }
        );
        assert_eq!(timer.last_delta(), &0.7);
        assert_eq!(
            timer.tick(0.8),
            Msg {
                should_spawn: true,
                should_fall: true,
                should_drift: true
            }
        );
        assert_eq!(timer.last_delta(), &0.8);
        assert_eq!(
            timer.tick(0.2),
            Msg {
                should_spawn: false,
                should_fall: false,
                should_drift: false
            }
        );
        assert_eq!(timer.last_delta(), &0.2);
    }
}
