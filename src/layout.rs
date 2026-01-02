use std::rc::Rc;

use crate::Component;

#[derive(Copy, Clone, Debug)]
pub enum Relative {
    NotMoved,
}

#[derive(Clone)]
pub struct Absolute<'a> {
    pub content: Rc<Component<'a>>,
}
