use std::io::{stdout, StdoutLock, Write};

use crossterm::{
    cursor,
    style::Print,
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use squalid::_d;

use crate::{
    size, take_over_screen, Component, Cursor, Error, Size, TakeOverScreenGuard, Text, TextChild,
};

pub struct Renderer {
    pub take_over_screen_guard: TakeOverScreenGuard,
    pub stdout: StdoutLock<'static>,
    pub size: Size,
    pub rendered_cursor_position_in_this_render: Option<Position>,
}

impl Renderer {
    pub fn try_new() -> Result<Self, Error> {
        Ok(Self {
            take_over_screen_guard: take_over_screen()?,
            stdout: stdout().lock(),
            size: size()?,
            rendered_cursor_position_in_this_render: _d(),
        })
    }

    pub fn render(&mut self, component: Component) -> Result<(), Error> {
        self.rendered_cursor_position_in_this_render = _d();
        self.size = size()?;

        self.stdout
            .queue(Clear(ClearType::All))
            .map_err(|_| Error::Crossterm("clear failed".into()))?;
        self.stdout
            .queue(cursor::Hide)
            .map_err(|_| Error::Crossterm("hide failed".into()))?;
        self.stdout
            .queue(cursor::MoveTo(0, 0))
            .map_err(|_| Error::Crossterm("move to failed".into()))?;

        match component {
            Component::Text(text) => self.render_text(text)?,
        }

        if let Some(cursor_position) = self.rendered_cursor_position_in_this_render {
            self.stdout
                .queue(cursor::MoveTo(cursor_position.column, cursor_position.row))
                .map_err(|_| Error::Crossterm("move to failed".into()))?;
            self.stdout
                .queue(cursor::Show)
                .map_err(|_| Error::Crossterm("show failed".into()))?;
        }

        self.stdout
            .flush()
            .map_err(|_| Error::Crossterm("flush failed".into()))?;

        Ok(())
    }

    fn render_text(&mut self, text: Text) -> Result<(), Error> {
        for child in text.children {
            match child {
                TextChild::Text(text) => self.print_text(&text)?,
                TextChild::Nested(text) => self.render_text(*text)?,
                TextChild::Cursor(cursor) => self.render_cursor(cursor)?,
            }
        }

        Ok(())
    }

    fn print_text(&mut self, text: &str) -> Result<(), Error> {
        self.stdout
            .queue(Print(text))
            .map_err(|_| Error::Crossterm("print failed".into()))?;

        Ok(())
    }

    fn render_cursor(&mut self, cursor: Cursor) -> Result<(), Error> {
        if self.rendered_cursor_position_in_this_render.is_some() {
            return Err(Error::RenderedCursorMoreThanOnce);
        }
        self.rendered_cursor_position_in_this_render = Some(unimplemented!());

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub row: u16,
    pub column: u16,
}
