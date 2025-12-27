use crate::Text;

pub enum Component {
    Text(Text),
}

impl From<Text> for Component {
    fn from(value: Text) -> Self {
        Self::Text(value)
    }
}
