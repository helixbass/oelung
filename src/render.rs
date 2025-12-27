use std::io::{stdout, StdoutLock, Write};

use crossterm::{
    cursor,
    style::Print,
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use squalid::_d;

use crate::{
    size, take_over_screen, Component, Cursor, Error, Offset, Size, TakeOverScreenGuard, Text,
    TextChild,
};

pub struct Renderer {
    pub take_over_screen_guard: TakeOverScreenGuard,
    pub stdout: StdoutLock<'static>,
    pub size: Size,
    pub rendered_cursor_position_in_this_render: Option<Position>,
    pub staged_this_render: Option<Staged>,
}

impl Renderer {
    pub fn try_new() -> Result<Self, Error> {
        Ok(Self {
            take_over_screen_guard: take_over_screen()?,
            stdout: stdout().lock(),
            size: size()?,
            rendered_cursor_position_in_this_render: _d(),
            staged_this_render: _d(),
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

        let grid = Grid {
            left: 0,
            top: 0,
            width: self.size.width,
            height: self.size.height,
        };
        let mut rendering_context = RenderingContext::new(self, grid);
        rendering_context.render(component)?;
        let RenderingContext {
            staged,
            rendered_cursor_position,
            ..
        } = rendering_context;
        self.staged_this_render = Some(staged);
        if let Some(rendered_cursor_position) = rendered_cursor_position {
            if self.rendered_cursor_position_in_this_render.is_some() {
                return Err(Error::RenderedCursorMoreThanOnce);
            }
            self.rendered_cursor_position_in_this_render = Some(rendered_cursor_position);
        }

        self.render_staged()?;

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

    fn render_staged(&mut self) -> Result<(), Error> {
        let staged = self.staged_this_render.as_ref().unwrap();
        for (row_index, row) in staged.lines.iter().enumerate() {
            self.stdout
                .queue(Print(row))
                .map_err(|_| Error::Crossterm("print failed".into()))?;

            if row_index < staged.lines.len() - 1 {
                self.stdout
                    .queue(Print("\r\n"))
                    .map_err(|_| Error::Crossterm("print failed".into()))?;
            }
        }

        Ok(())
    }
}

pub struct RenderingContext<'a> {
    pub renderer: &'a mut Renderer,
    pub grid: Grid,
    pub staged: Staged,
    pub rendered_cursor_position: Option<Position>,
    // pub current_line_number: Option<usize>,
}

impl<'a> RenderingContext<'a> {
    pub fn new(renderer: &'a mut Renderer, grid: Grid) -> Self {
        Self {
            renderer,
            grid,
            staged: _d(),
            rendered_cursor_position: _d(),
            // current_line_number: _d(),
        }
    }

    pub fn render(&mut self, component: Component) -> Result<(), Error> {
        match component {
            Component::Text(text) => {
                self.staged.lines.push(_d());
                self.render_text(text, 0)?;
            }
            Component::FlexColumn(flex_column) => unimplemented!(),
        }

        Ok(())
    }

    pub fn render_text(&mut self, text: Text, line_num: usize) -> Result<(), Error> {
        for child in text.children {
            match child {
                TextChild::Text(text) => self.print_text(&text, line_num)?,
                TextChild::Nested(text) => self.render_text(*text, line_num)?,
                TextChild::Cursor(cursor) => self.render_cursor(cursor)?,
            }
        }

        Ok(())
    }

    fn print_text(&mut self, text: &str, line_num: usize) -> Result<(), Error> {
        self.staged.lines[line_num].push_str(text);

        Ok(())
    }

    fn render_cursor(&mut self, cursor: Cursor) -> Result<(), Error> {
        if self.rendered_cursor_position.is_some() {
            return Err(Error::RenderedCursorMoreThanOnce);
        }
        self.rendered_cursor_position = Some(match cursor {
            Cursor::Relative(Offset { x, y }) => Position { row: y, column: x },
        });

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub row: u16,
    pub column: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    pub left: u16,
    pub top: u16,
    pub height: u16,
    pub width: u16,
}

#[derive(Default)]
pub struct Staged {
    pub lines: Vec<String>,
}
