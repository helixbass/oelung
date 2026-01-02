use std::rc::Rc;

use crate::{Absolute, FlexColumn, Grid, Text};

#[derive(Clone)]
pub enum Component<'a> {
    Text(Text<'a>),
    FlexColumn(FlexColumn<'a>),
    Absolute(Absolute<'a>),
    Component(Rc<dyn ComponentInterface + 'a>),
}

impl<'a> Component<'a> {
    pub fn into_component(self) -> Rc<dyn ComponentInterface + 'a> {
        match self {
            Self::Component(component) => component,
            _ => panic!("expected component"),
        }
    }

    pub fn as_component(&self) -> &Rc<dyn ComponentInterface + 'a> {
        match self {
            Self::Component(component) => component,
            _ => panic!("expected component"),
        }
    }

    pub fn as_absolute(&self) -> &Absolute<'a> {
        match self {
            Self::Absolute(absolute) => absolute,
            _ => panic!("expected absolute"),
        }
    }

    pub fn flex_grow(&self) -> Option<f64> {
        match self {
            Self::Text(_) => None,
            Self::FlexColumn(flex_column) => flex_column.flex_grow(),
            Self::Absolute(_) => unreachable!(),
            Self::Component(component) => component.flex_grow(),
        }
    }

    pub fn height(&self) -> Option<u16> {
        match self {
            Self::Text(text) => text.height(),
            Self::FlexColumn(_) => None,
            Self::Absolute(_) => unreachable!(),
            Self::Component(component) => component.height(),
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

impl<'a> From<Absolute<'a>> for Component<'a> {
    fn from(value: Absolute<'a>) -> Self {
        Self::Absolute(value)
    }
}

pub trait ComponentInterface {
    fn render(&self, grid: Grid) -> Result<Component<'_>, anyhow::Error>;

    fn flex_grow(&self) -> Option<f64> {
        None
    }

    fn height(&self) -> Option<u16> {
        None
    }
}
