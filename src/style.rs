use crossterm::style::Color;
use derive_builder::Builder;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Builder)]
pub struct Style {
    #[builder(setter(strip_option), default)]
    pub color: Option<Color>,
    #[builder(setter(strip_option), default)]
    pub background_color: Option<Color>,
}
