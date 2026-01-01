use crate::{FlexColumn, Grid, Text};

pub enum Component<'a> {
    Text(Text<'a>),
    FlexColumn(FlexColumn<'a>),
    Component(Box<dyn ComponentInterface + 'a>),
}

impl<'a> Component<'a> {
    pub fn into_component(self) -> Box<dyn ComponentInterface + 'a> {
        match self {
            Self::Component(component) => component,
            _ => panic!("expected component"),
        }
    }

    pub fn as_component(&self) -> &Box<dyn ComponentInterface + 'a> {
        match self {
            Self::Component(component) => component,
            _ => panic!("expected component"),
        }
    }
}

impl<'a> From<Text<'a>> for Component<'a> {
    fn from(value: Text<'a>) -> Self {
        Self::Text(value)
    }
}

impl<'a> From<FlexColumn<'a>> for Component<'a> {
    fn from(value: FlexColumn<'a>) -> Self {
        Self::FlexColumn(value)
    }
}

pub trait ComponentInterface {
    fn flex_grow(&self) -> Option<f64> {
        None
    }

    fn height(&self) -> Option<u16> {
        None
    }

    fn render<'a: 'b, 'b>(&'b self, grid: Grid) -> Result<Component<'a>, anyhow::Error>;
}

impl<'a> ComponentInterface for Component<'a> {
    fn flex_grow(&self) -> Option<f64> {
        match self {
            Self::Text(text) => text.flex_grow(),
            Self::FlexColumn(flex_column) => flex_column.flex_grow(),
            Self::Component(component) => component.flex_grow(),
        }
    }

    fn height(&self) -> Option<u16> {
        match self {
            Self::Text(text) => text.height(),
            Self::FlexColumn(flex_column) => flex_column.height(),
            Self::Component(component) => component.height(),
        }
    }

    fn render<'b: 'c, 'c>(&'c self, grid: Grid) -> Result<Component<'b>, anyhow::Error> {
        match self {
            Self::Text(text) => text.render(grid),
            Self::FlexColumn(flex_column) => flex_column.render(grid),
            Self::Component(component) => component.render(grid),
        }
    }
}
