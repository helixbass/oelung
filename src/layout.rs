use std::rc::Rc;

use crate::Component;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Relative {
    NotMoved,
}

#[derive(Clone)]
pub struct Absolute<'a> {
    pub content: Rc<Component<'a>>,
}

impl<'a> Absolute<'a> {
    pub fn new(content: Rc<Component<'a>>) -> Self {
        Self { content }
    }
}
