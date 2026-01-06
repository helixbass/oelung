use std::fmt::Display;
use std::io::Write;

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
    Memory(BackendMemory),
}

impl BackendInterface for Backend {
    fn size(&self) -> Result<Size, Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.size(),
            Self::Memory(memory) => memory.size(),
        }
    }

    fn queue_hide_cursor(&mut self) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_hide_cursor(),
            Self::Memory(memory) => memory.queue_hide_cursor(),
        }
    }

    fn queue_move_cursor(
        &mut self,
        column: RowOrColumnNumber,
        row: RowOrColumnNumber,
    ) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_move_cursor(column, row),
            Self::Memory(memory) => memory.queue_move_cursor(column, row),
        }
    }

    fn queue_show_cursor(&mut self) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_show_cursor(),
            Self::Memory(memory) => memory.queue_show_cursor(),
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.flush(),
            Self::Memory(memory) => memory.flush(),
        }
    }

    fn queue_set_foreground_color(&mut self, color: Color) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_set_foreground_color(color),
            Self::Memory(memory) => memory.queue_set_foreground_color(color),
        }
    }

    fn queue_set_background_color(&mut self, color: Color) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_set_background_color(color),
            Self::Memory(memory) => memory.queue_set_background_color(color),
        }
    }

    fn queue_print<TDisplay: Display>(&mut self, value: TDisplay) -> Result<(), Error> {
        match self {
            Self::Crossterm(crossterm) => crossterm.queue_print(value),
            Self::Memory(memory) => memory.queue_print(value),
        }
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
    pub foreground_color: Option<Color>,
    pub background_color: Option<Color>,
}

impl BackendMemory {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            cursor_position: _d(),
            foreground_color: _d(),
            background_color: _d(),
        }
    }
}

impl BackendInterface for BackendMemory {
    fn size(&self) -> Result<Size, Error> {
        Ok(self.size)
    }

    fn queue_hide_cursor(&mut self) -> Result<(), Error> {
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
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn queue_set_foreground_color(&mut self, color: Color) -> Result<(), Error> {
        self.foreground_color = Some(color);

        Ok(())
    }

    fn queue_set_background_color(&mut self, color: Color) -> Result<(), Error> {
        self.background_color = Some(color);

        Ok(())
    }

    fn queue_print<TDisplay: Display>(&mut self, value: TDisplay) -> Result<(), Error> {
        unimplemented!()
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
