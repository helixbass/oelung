use crossterm::style::Color;
use derive_builder::Builder;

#[derive(Builder)]
pub struct Style {
    #[builder(setter(strip_option), default)]
    pub color: Option<Color>,
}
