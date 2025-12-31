use std::fmt::Display;

use crossterm::style::Color;

use smallvec::SmallVec;
use smol_str::{SmolStr, ToSmolStr};

use crate::{Component, ComponentInterface, Cursor, Error, Grid, Style, StyleBuilder};

pub struct Text<'a, 'b> {
    pub children: TextChildren<'a, 'b>,
    pub cursor: Option<Cursor>,
    pub style: Option<Style>,
}

impl<'a, 'b> ComponentInterface<'a> for Text<'a, 'b> {
    fn height(&self) -> Option<u16> {
        Some(1)
    }

    fn render(&self, _grid: Grid) -> Result<Component<'a, 'static>, anyhow::Error> {
        unreachable!()
    }
}

#[derive(Default)]
pub struct TextBuilder<'a, 'b> {
    pub children: TextChildren<'a, 'b>,
    pub cursor: Option<Cursor>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
}

impl<'a, 'b> TextBuilder<'a, 'b> {
    pub fn text_child(mut self, child: impl Display) -> Self {
        self.children.push(child.to_smolstr().into());
        self
    }

    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn nested_child(mut self, child: Text<'a, 'b>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn nested_component_child(mut self, child: Box<dyn ComponentInterface<'a> + 'b>) -> Self {
        self.children.push(TextChild::NestedComponent(child));
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn build(self) -> Result<Text<'a, 'b>, Error> {
        if self.children.is_empty() {
            return Err(Error::TextBuilder("empty children".into()));
        }
        Ok(Text {
            children: self.children,
            cursor: self.cursor,
            style: {
                let mut style = StyleBuilder::default();
                if let Some(color) = self.color {
                    style.color(color);
                }
                if let Some(background_color) = self.background_color {
                    style.background_color(background_color);
                }
                Some(style.build().unwrap())
            },
        })
    }
}

pub type TextChildren<'a, 'b> = SmallVec<[TextChild<'a, 'b>; 10]>;

pub enum TextChild<'a, 'b> {
    Nested(Box<Text<'a, 'b>>),
    NestedComponent(Box<dyn ComponentInterface<'a> + 'b>),
    Text(SmolStr),
}

impl<'a, 'b> From<Text<'a, 'b>> for TextChild<'a, 'b> {
    fn from(value: Text<'a, 'b>) -> Self {
        Self::Nested(Box::new(value))
    }
}

impl<'a, 'b> From<SmolStr> for TextChild<'a, 'b> {
    fn from(value: SmolStr) -> Self {
        Self::Text(value)
    }
}
