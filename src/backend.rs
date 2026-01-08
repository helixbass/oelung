use std::cell::RefCell;
use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;

use crossterm::{
    cursor,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use squalid::{EverythingExt, _d};
use tracing::instrument;

use crate::{take_over_screen, Error, TakeOverScreenGuard};

pub enum Backend {
    Crossterm(BackendCrossterm),
    Memory(Rc<RefCell<BackendMemory>>),
}

impl BackendInterface for Backend {
    fn size(&self) -> Result<Size, Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.size(),
            Self::Memory(memory) => memory.borrow().size(),
        }
    }

    fn queue_hide_cursor(&mut self) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_hide_cursor(),
            Self::Memory(memory) => memory.borrow_mut().queue_hide_cursor(),
        }
    }

    fn queue_move_cursor(
        &mut self,
        column: RowOrColumnNumber,
        row: RowOrColumnNumber,
    ) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_move_cursor(column, row),
            Self::Memory(memory) => memory.borrow_mut().queue_move_cursor(column, row),
        }
    }

    fn queue_show_cursor(&mut self) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_show_cursor(),
            Self::Memory(memory) => memory.borrow_mut().queue_show_cursor(),
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.flush(),
            Self::Memory(memory) => memory.borrow_mut().flush(),
        }
    }

    fn queue_set_foreground_color(&mut self, color: Color) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_set_foreground_color(color),
            Self::Memory(memory) => memory.borrow_mut().queue_set_foreground_color(color),
        }
    }

    fn queue_set_background_color(&mut self, color: Color) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_set_background_color(color),
            Self::Memory(memory) => memory.borrow_mut().queue_set_background_color(color),
        }
    }

    fn queue_print<TDisplay: Display>(&mut self, value: TDisplay) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_print(value),
            Self::Memory(memory) => memory.borrow_mut().queue_print(value),
        }
    }

    fn finished_render(&mut self) {
        match self {
            Self::Crossterm(crossterm) => crossterm.finished_render(),
            Self::Memory(memory) => memory.borrow_mut().finished_render(),
        }
    }
}

impl From<BackendCrossterm> for Backend {
    fn from(value: BackendCrossterm) -> Self {
        Self::Crossterm(value)
    }
}

impl From<Rc<RefCell<BackendMemory>>> for Backend {
    fn from(value: Rc<RefCell<BackendMemory>>) -> Self {
        Self::Memory(value)
    }
}

pub struct BackendCrossterm {
    pub take_over_screen_guard: TakeOverScreenGuard,
}

impl BackendCrossterm {
    pub fn try_new() -> Result<Self, Error> {
        let mut take_over_screen_guard = take_over_screen()?;

        take_over_screen_guard
            .stdout
            .execute(Clear(ClearType::All))
            .map_err(|_| Error::Crossterm("clear failed".into()))?;

        Ok(Self {
            take_over_screen_guard,
        })
    }
}

impl BackendInterface for BackendCrossterm {
    #[instrument(level = "trace", skip(self))]
    fn size(&self) -> Result<Size, Error> {
        Ok(terminal::size()
            .map_err(|_| Error::Crossterm("size failed".into()))?
            .thrush(|size| Size {
                height: size.1,
                width: size.0,
            }))
    }

    fn queue_hide_cursor(&mut self) -> Result<(), Error> {
        self.take_over_screen_guard
            .stdout
            .queue(cursor::Hide)
            .map_err(|_| Error::Crossterm("hide failed".into()))?;

        Ok(())
    }

    fn queue_move_cursor(
        &mut self,
        column: RowOrColumnNumber,
        row: RowOrColumnNumber,
    ) -> Result<(), Error> {
        self.take_over_screen_guard
            .stdout
            .queue(cursor::MoveTo(column, row))
            .map_err(|_| Error::Crossterm("move to failed".into()))?;

        Ok(())
    }

    fn queue_show_cursor(&mut self) -> Result<(), Error> {
        self.take_over_screen_guard
            .stdout
            .queue(cursor::Show)
            .map_err(|_| Error::Crossterm("show failed".into()))?;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.take_over_screen_guard
            .stdout
            .flush()
            .map_err(|_| Error::Crossterm("flush failed".into()))?;

        Ok(())
    }

    fn queue_set_foreground_color(&mut self, color: Color) -> Result<(), Error> {
        self.take_over_screen_guard
            .stdout
            .queue(SetForegroundColor(color))
            .map_err(|_| Error::Crossterm("set foreground color failed".into()))?;

        Ok(())
    }

    fn queue_set_background_color(&mut self, color: Color) -> Result<(), Error> {
        self.take_over_screen_guard
            .stdout
            .queue(SetBackgroundColor(color))
            .map_err(|_| Error::Crossterm("set background color failed".into()))?;

        Ok(())
    }

    fn queue_print<TDisplay: Display>(&mut self, value: TDisplay) -> Result<(), Error> {
        self.take_over_screen_guard
            .stdout
            .queue(Print(value))
            .map_err(|_| Error::Crossterm("print failed".into()))?;

        Ok(())
    }
}

pub struct BackendMemory {
    pub size: Size,
    pub cursor_position: Option<Position>,
    pub foreground_color: Color,
    pub background_color: Color,
    pub grid: Vec<Vec<Cell>>,
    pub is_cursor_shown: bool,
    pub rendered_grids: Vec<Vec<Vec<Cell>>>,
}

impl BackendMemory {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            cursor_position: _d(),
            foreground_color: Color::Reset,
            background_color: Color::Reset,
            grid: vec![vec![_d(); usize::from(size.width)]; usize::from(size.height)],
            is_cursor_shown: false,
            rendered_grids: _d(),
        }
    }

    pub fn current_cursor_position(&self) -> Option<Position> {
        self.is_cursor_shown.then(|| self.cursor_position.unwrap())
    }
}

impl BackendInterface for BackendMemory {
    fn size(&self) -> Result<Size, Error> {
        Ok(self.size)
    }

    fn queue_hide_cursor(&mut self) -> Result<(), Error> {
        self.is_cursor_shown = false;

        Ok(())
    }

    fn queue_move_cursor(
        &mut self,
        column: RowOrColumnNumber,
        row: RowOrColumnNumber,
    ) -> Result<(), Error> {
        self.cursor_position = Some(Position { row, column });

        Ok(())
    }

    fn queue_show_cursor(&mut self) -> Result<(), Error> {
        self.is_cursor_shown = true;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn queue_set_foreground_color(&mut self, color: Color) -> Result<(), Error> {
        self.foreground_color = color;

        Ok(())
    }

    fn queue_set_background_color(&mut self, color: Color) -> Result<(), Error> {
        self.background_color = color;

        Ok(())
    }

    fn queue_print<TDisplay: Display>(&mut self, value: TDisplay) -> Result<(), Error> {
        let cursor_position = self.cursor_position.unwrap();
        let value = value.to_string();
        let value_len = value.len();
        for (index, ch) in value.chars().enumerate() {
            self.grid[usize::from(cursor_position.row)]
                [usize::from(cursor_position.column) + index] = Cell {
                content: ch,
                foreground_color: self.foreground_color,
                background_color: self.background_color,
            };
        }
        self.cursor_position = Some(Position {
            row: cursor_position.row,
            column: cursor_position.column + u16::try_from(value_len).unwrap(),
        });

        Ok(())
    }

    fn finished_render(&mut self) {
        self.rendered_grids.push(self.grid.clone());
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub content: char,
    pub foreground_color: Color,
    pub background_color: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: ' ',
            foreground_color: Color::Reset,
            background_color: Color::Reset,
        }
    }
}

pub trait BackendInterface {
    fn size(&self) -> Result<Size, Error>;
    fn queue_hide_cursor(&mut self) -> Result<(), Error>;
    fn queue_move_cursor(
        &mut self,
        column: RowOrColumnNumber,
        row: RowOrColumnNumber,
    ) -> Result<(), Error>;
    fn queue_show_cursor(&mut self) -> Result<(), Error>;
    fn flush(&mut self) -> Result<(), Error>;
    fn queue_set_foreground_color(&mut self, color: Color) -> Result<(), Error>;
    fn queue_set_background_color(&mut self, color: Color) -> Result<(), Error>;
    fn queue_print<TDisplay: Display>(&mut self, value: TDisplay) -> Result<(), Error>;
    fn finished_render(&mut self) {}
}

pub type RowOrColumnNumber = u16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Size {
    pub height: RowOrColumnNumber,
    pub width: RowOrColumnNumber,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub row: RowOrColumnNumber,
    pub column: RowOrColumnNumber,
}
