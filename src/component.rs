use crate::{FlexColumn, Grid, Text};

pub enum Component {
    Text(Text),
    FlexColumn(FlexColumn),
    Component(Box<dyn ComponentInterface>),
}

impl From<Text> for Component {
    fn from(value: Text) -> Self {
        Self::Text(value)
    }
}

impl From<FlexColumn> for Component {
    fn from(value: FlexColumn) -> Self {
        Self::FlexColumn(value)
    }
}

pub enum ComponentOrFragment {
    Component(Component),
    Fragment(Fragment),
}

pub struct Fragment {
    pub children: Vec<Component>,
}

impl From<Fragment> for ComponentOrFragment {
    fn from(value: Fragment) -> Self {
        Self::Fragment(value)
    }
}

impl From<Component> for ComponentOrFragment {
    fn from(value: Component) -> Self {
        Self::Component(value)
    }
}

pub trait ComponentInterface {
    fn flex_grow(&self) -> Option<f64> {
        None
    }

    fn height(&self) -> Option<u16> {
        None
    }

    fn render(&self, grid: Grid) -> Result<ComponentOrFragment, anyhow::Error>;
}

impl ComponentInterface for Component {
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

    fn render(&self, grid: Grid) -> Result<ComponentOrFragment, anyhow::Error> {
        match self {
            Self::Text(text) => text.render(grid),
            Self::FlexColumn(flex_column) => flex_column.render(grid),
            Self::Component(component) => component.render(grid),
        }
    }
}
