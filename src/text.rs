use std::fmt::Display;

use smallvec::SmallVec;
use smol_str::{SmolStr, ToSmolStr};

use crate::{Component, ComponentInterface, Cursor, Error, Grid};

pub struct Text {
    pub children: TextChildren,
    pub cursor: Option<Cursor>,
}

impl ComponentInterface for Text {
    fn height(&self) -> Option<u16> {
        Some(1)
    }

    fn render<'a>(&self, _grid: Grid) -> Result<Component<'a>, anyhow::Error> {
        unreachable!()
    }
}

#[derive(Default)]
pub struct TextBuilder {
    pub children: TextChildren,
    pub cursor: Option<Cursor>,
}

impl TextBuilder {
    pub fn text_child(mut self, child: impl Display) -> Self {
        self.children.push(child.to_smolstr().into());
        self
    }

    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn nested_child(mut self, child: Text) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn build(self) -> Result<Text, Error> {
        if self.children.is_empty() {
            return Err(Error::TextBuilder("empty children".into()));
        }
        Ok(Text {
            children: self.children,
            cursor: self.cursor,
        })
    }
}

pub type TextChildren = SmallVec<[TextChild; 10]>;

pub enum TextChild {
    Nested(Box<Text>),
    Text(SmolStr),
}

impl From<Text> for TextChild {
    fn from(value: Text) -> Self {
        Self::Nested(Box::new(value))
    }
}

impl From<SmolStr> for TextChild {
    fn from(value: SmolStr) -> Self {
        Self::Text(value)
    }
}
