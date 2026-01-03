use crate::{Component, Cursor, Error, Overflow, Relative};

#[derive(Clone)]
pub struct FlexColumn<'a> {
    pub children: Vec<Component<'a>>,
    pub flex_grow: Option<f64>,
    pub relative: Option<Relative>,
    pub cursor: Option<Cursor>,
    pub overflow_y: Option<Overflow>,
}

impl<'a> FlexColumn<'a> {
    pub fn flex_grow(&self) -> Option<f64> {
        self.flex_grow
    }
}

#[derive(Default)]
pub struct FlexColumnBuilder<'a> {
    pub children: Vec<Component<'a>>,
    pub flex_grow: Option<f64>,
    pub relative: Option<Relative>,
    pub cursor: Option<Cursor>,
    pub overflow_y: Option<Overflow>,
    pub has_called_children: bool,
}

impl<'a> FlexColumnBuilder<'a> {
    pub fn child(mut self, child: impl Into<Component<'a>>) -> Self {
        if self.has_called_children {
            panic!("Can't use both `.children()` and `.child()`");
        }
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: Vec<Component<'a>>) -> Self {
        assert!(
            self.children.is_empty(),
            "Can't use both `.children()` and `.child()`"
        );
        self.children = children;
        self.has_called_children = true;
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

    pub fn overflow_y(mut self, overflow_y: Overflow) -> Self {
        self.overflow_y = Some(overflow_y);
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
            overflow_y: self.overflow_y,
        })
    }
}
