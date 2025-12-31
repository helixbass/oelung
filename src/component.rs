use crate::{FlexColumn, Grid, Text};

pub enum Component<'a, 'b> {
    Text(Text<'a, 'b>),
    FlexColumn(FlexColumn<'a, 'b>),
    Component(Box<dyn ComponentInterface<'a> + 'b>),
}

impl<'a, 'b> Component<'a, 'b> {
    pub fn into_component(self) -> Box<dyn ComponentInterface<'a> + 'b> {
        match self {
            Self::Component(component) => component,
            _ => panic!("expected component"),
        }
    }

    pub fn as_component(&self) -> &Box<dyn ComponentInterface<'a> + 'b> {
        match self {
            Self::Component(component) => component,
            _ => panic!("expected component"),
        }
    }
}

impl<'a, 'b> From<Text<'a, 'b>> for Component<'a, 'b> {
    fn from(value: Text<'a, 'b>) -> Self {
        Self::Text(value)
    }
}

impl<'a, 'b> From<FlexColumn<'a, 'b>> for Component<'a, 'b> {
    fn from(value: FlexColumn<'a, 'b>) -> Self {
        Self::FlexColumn(value)
    }
}

pub trait ComponentInterface<'a> {
    fn flex_grow(&self) -> Option<f64> {
        None
    }

    fn height(&self) -> Option<u16> {
        None
    }

    fn render(&self, grid: Grid) -> Result<Component<'a, 'static>, anyhow::Error>;
}

impl<'a, 'b> ComponentInterface<'a> for Component<'a, 'b> {
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

    fn render(&self, grid: Grid) -> Result<Component<'a, 'static>, anyhow::Error> {
        match self {
            Self::Text(text) => text.render(grid),
            Self::FlexColumn(flex_column) => flex_column.render(grid),
            Self::Component(component) => component.render(grid),
        }
    }
}
