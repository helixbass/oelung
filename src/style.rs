use crossterm::style::Color;
use derive_builder::Builder;

#[derive(Copy, Clone, Debug, Default, Builder)]
pub struct Style {
    #[builder(setter(strip_option), default)]
    pub color: Option<Color>,
}
