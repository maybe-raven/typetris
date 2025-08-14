use web_sys::CssStyleDeclaration;

#[derive(Debug, Clone)]
pub(super) struct Swatch {
    pub(super) bg_color: String,
    pub(super) regular_block_color: String,
    pub(super) disabled_block_color: String,
    pub(super) success_color: String,
    pub(super) error_color: String,
}

impl Default for Swatch {
    fn default() -> Self {
        Self {
            bg_color: "black".into(),
            regular_block_color: "purple".into(),
            disabled_block_color: "gray".into(),
            success_color: "green".into(),
            error_color: "red".into(),
        }
    }
}

impl Swatch {
    #[inline]
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn extract(&mut self, style: CssStyleDeclaration) {
        self.regular_block_color = style.get_property_value("--color-primary").unwrap();
        self.disabled_block_color = style.get_property_value("--color-dark3").unwrap();
        self.bg_color = style.get_property_value("--color-dark2").unwrap();
        self.success_color = style.get_property_value("--color-success").unwrap();
        self.error_color = style.get_property_value("--color-error").unwrap();
    }
}
