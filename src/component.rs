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
