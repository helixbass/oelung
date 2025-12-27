use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::Error;

pub struct Text {
    pub children: TextChildren,
}

#[derive(Default)]
pub struct TextBuilder {
    pub children: TextChildren,
}

impl TextBuilder {
    pub fn text_child(&mut self, child: impl Into<SmolStr>) -> &mut Self {
        self.children.push(child.into().into());
        self
    }

    pub fn build(self) -> Result<Text, Error> {
        if self.children.is_empty() {
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
