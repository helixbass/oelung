use crate::{FlexColumn, Text};

pub enum Component {
    Text(Text),
    FlexColumn(FlexColumn),
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

pub trait ComponentInterface {
    fn flex_grow(&self) -> Option<f64> {
        None
    }

    fn height(&self) -> Option<u16> {
        None
    }
}

impl ComponentInterface for Component {
    fn flex_grow(&self) -> Option<f64> {
        match self {
            Self::Text(text) => text.flex_grow(),
            Self::FlexColumn(flex_column) => flex_column.flex_grow(),
        }
    }

    fn height(&self) -> Option<u16> {
        match self {
            Self::Text(text) => text.height(),
            Self::FlexColumn(flex_column) => flex_column.height(),
        }
    }
}
