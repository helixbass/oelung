use crate::{Component, ComponentInterface, Cursor, Error, Grid};

pub struct FlexColumn<'a, 'b> {
    pub children: Vec<Component<'a, 'b>>,
    pub flex_grow: Option<f64>,
    pub cursor: Option<Cursor>,
}

impl<'a, 'b> ComponentInterface<'a> for FlexColumn<'a, 'b> {
    fn flex_grow(&self) -> Option<f64> {
        self.flex_grow
    }

    fn render(&self, _grid: Grid) -> Result<Component<'a, 'static>, anyhow::Error> {
        unreachable!()
    }
}

#[derive(Default)]
pub struct FlexColumnBuilder<'a, 'b> {
    pub children: Vec<Component<'a, 'b>>,
    pub flex_grow: Option<f64>,
    pub cursor: Option<Cursor>,
}

impl<'a, 'b> FlexColumnBuilder<'a, 'b> {
    pub fn child(mut self, child: impl Into<Component<'a, 'b>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn flex_grow(mut self, value: impl Into<f64>) -> Self {
        self.flex_grow = Some(value.into());
        self
    }

    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn build(self) -> Result<FlexColumn<'a, 'b>, Error> {
        if self.children.is_empty() {
            return Err(Error::FlexColumnBuilder("empty children".into()));
        }
        Ok(FlexColumn {
            children: self.children,
            flex_grow: self.flex_grow,
            cursor: self.cursor,
        })
    }
}
