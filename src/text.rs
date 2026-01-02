use std::fmt::Display;
use std::rc::Rc;

use crossterm::style::Color;

use smallvec::SmallVec;
use smol_str::{SmolStr, ToSmolStr};

use crate::{ComponentInterface, Cursor, Error, Style, StyleBuilder};

#[derive(Clone)]
pub struct Text<'a> {
    pub children: TextChildren<'a>,
    pub cursor: Option<Cursor>,
    pub style: Option<Style>,
}

impl<'a> Text<'a> {
    pub fn height(&self) -> Option<u16> {
        Some(1)
    }
}

#[derive(Default)]
pub struct TextBuilder<'a> {
    pub children: TextChildren<'a>,
    pub cursor: Option<Cursor>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
}

impl<'a> TextBuilder<'a> {
    pub fn text_child(mut self, child: impl Display) -> Self {
        self.children.push(child.to_smolstr().into());
        self
    }

    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn nested_child(mut self, child: Text<'a>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn nested_component_child(mut self, child: Rc<dyn ComponentInterface + 'a>) -> Self {
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

    pub fn build(self) -> Result<Text<'a>, Error> {
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

pub type TextChildren<'a> = SmallVec<TextChild<'a>, 10>;

#[derive(Clone)]
pub enum TextChild<'a> {
    Nested(Rc<Text<'a>>),
    NestedComponent(Rc<dyn ComponentInterface + 'a>),
    Text(SmolStr),
}

impl<'a> From<Text<'a>> for TextChild<'a> {
    fn from(value: Text<'a>) -> Self {
        Self::Nested(Rc::new(value))
    }
}

impl<'a> From<SmolStr> for TextChild<'a> {
    fn from(value: SmolStr) -> Self {
        Self::Text(value)
    }
}
