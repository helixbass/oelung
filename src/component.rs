use crate::{FlexColumn, Grid, Text};

pub enum Component<'a> {
    Text(Text),
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

impl From<Text> for Component<'_> {
    fn from(value: Text) -> Self {
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

    fn render(&self, grid: Grid) -> Result<Component<'_>, anyhow::Error>;
}

impl ComponentInterface for Component<'_> {
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

    fn render(&self, grid: Grid) -> Result<Component, anyhow::Error> {
        match self {
            Self::Text(text) => text.render(grid),
            Self::FlexColumn(flex_column) => flex_column.render(grid),
            Self::Component(component) => component.render(grid),
        }
    }
}
