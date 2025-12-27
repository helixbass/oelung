use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{Component, ComponentInterface, Cursor, Error, Grid};

pub struct Text {
    pub children: TextChildren,
}

impl ComponentInterface for Text {
    fn height(&self) -> Option<u16> {
        Some(1)
    }

    fn render(&self, _grid: Grid) -> Result<Component, anyhow::Error> {
        unreachable!()
    }
}

#[derive(Default)]
pub struct TextBuilder {
    pub children: TextChildren,
    pub has_seen_non_cursor_child: bool,
}

impl TextBuilder {
    pub fn text_child(mut self, child: impl Into<SmolStr>) -> Self {
        self.has_seen_non_cursor_child = true;
        self.children.push(child.into().into());
        self
    }

    pub fn cursor_child(mut self, child: Cursor) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn build(self) -> Result<Text, Error> {
        if !self.has_seen_non_cursor_child {
            return Err(Error::TextBuilder("empty children".into()));
        }
        Ok(Text {
            children: self.children,
        })
    }
}

pub type TextChildren = SmallVec<[TextChild; 10]>;

pub enum TextChild {
    Nested(Box<Text>),
    Text(SmolStr),
    Cursor(Cursor),
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

impl From<Cursor> for TextChild {
    fn from(value: Cursor) -> Self {
        Self::Cursor(value)
    }
}
