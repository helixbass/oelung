use crate::{Component, ComponentInterface, Cursor, Error, Grid, Relative};

pub struct FlexColumn<'a> {
    pub children: Vec<Component<'a>>,
    pub flex_grow: Option<f64>,
    pub relative: Option<Relative>,
    pub cursor: Option<Cursor>,
}

impl<'a> ComponentInterface for FlexColumn<'a> {
    fn flex_grow(&self) -> Option<f64> {
        self.flex_grow
    }

    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        unreachable!()
    }
}

#[derive(Default)]
pub struct FlexColumnBuilder<'a> {
    pub children: Vec<Component<'a>>,
    pub flex_grow: Option<f64>,
    pub relative: Option<Relative>,
    pub cursor: Option<Cursor>,
}

impl<'a> FlexColumnBuilder<'a> {
    pub fn child(mut self, child: impl Into<Component<'a>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn flex_grow(mut self, flex_grow: impl Into<f64>) -> Self {
        self.flex_grow = Some(flex_grow.into());
        self
    }

    pub fn relative(mut self, relative: Relative) -> Self {
        self.relative = Some(relative);
        self
    }

    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn build(self) -> Result<FlexColumn<'a>, Error> {
        if self.children.is_empty() {
            return Err(Error::FlexColumnBuilder("empty children".into()));
        }
        Ok(FlexColumn {
            children: self.children,
            flex_grow: self.flex_grow,
            relative: self.relative,
            cursor: self.cursor,
        })
    }
}
