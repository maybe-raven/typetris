use getset::WithSetters;

#[derive(Debug, Clone, Copy, WithSetters)]
pub struct Settings {
    #[getset(set_with = "pub")]
    pub width: u8,
    #[getset(set_with = "pub")]
    pub height: u8,
    #[getset(set_with = "pub")]
    pub starts_with_one: bool,
    #[getset(set_with = "pub")]
    pub fall_interval: f64,
    #[getset(set_with = "pub")]
    pub spawn_interval: f64,
    #[getset(set_with = "pub")]
    pub drift_interval: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            width: 12,
            height: 16,
            starts_with_one: true,
            spawn_interval: 4_000.0,
            fall_interval: 100.0,
            drift_interval: 8,
        }
    }
}
