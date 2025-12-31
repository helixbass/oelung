#[derive(Copy, Clone, Debug)]
pub enum Cursor {
    Relative(Offset),
}

impl Cursor {
    pub fn relative() -> CursorRelativeBuilderEmpty {
        CursorRelativeBuilderEmpty::default()
    }
}

#[derive(Default)]
pub struct CursorRelativeBuilderEmpty;

impl CursorRelativeBuilderEmpty {
    pub fn x(self, x: u16) -> CursorRelativeBuilderHasX {
        CursorRelativeBuilderHasX(x)
    }

    pub fn y(self, y: u16) -> CursorRelativeBuilderHasY {
        CursorRelativeBuilderHasY(y)
    }
}

pub struct CursorRelativeBuilderHasX(u16);

impl CursorRelativeBuilderHasX {
    pub fn y(self, y: u16) -> Cursor {
        Cursor::Relative(Offset { x: self.0, y })
    }
}

pub struct CursorRelativeBuilderHasY(u16);

impl CursorRelativeBuilderHasY {
    pub fn x(self, x: u16) -> Cursor {
        Cursor::Relative(Offset { x, y: self.0 })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Offset {
    pub x: u16,
    pub y: u16,
}
