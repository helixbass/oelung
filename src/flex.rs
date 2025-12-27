use crate::{Component, Error};

pub struct FlexColumn {
    pub children: Vec<Component>,
    pub flex_grow: Option<f64>,
}

#[derive(Default)]
pub struct FlexColumnBuilder {
    pub children: Vec<Component>,
    pub flex_grow: Option<f64>,
}

impl FlexColumnBuilder {
    pub fn child(mut self, child: impl Into<Component>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn flex_grow(mut self, value: impl Into<f64>) -> Self {
        self.flex_grow = Some(value.into());
        self
    }

    pub fn build(self) -> Result<FlexColumn, Error> {
        if self.children.is_empty() {
            return Err(Error::FlexColumnBuilder("empty children".into()));
        }
        Ok(FlexColumn {
            children: self.children,
            flex_grow: self.flex_grow,
        })
    }
}
