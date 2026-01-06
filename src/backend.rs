use crossterm::{
    cursor,
    terminal::{self, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use squalid::EverythingExt;
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
}

pub struct BackendMemory {
    pub size: Size,
}

impl BackendMemory {
    pub fn new(size: Size) -> Self {
        Self { size }
    }
}

impl BackendInterface for BackendMemory {
    fn size(&self) -> Result<Size, Error> {
        Ok(self.size)
    }

    fn queue_hide_cursor(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

pub trait BackendInterface {
    fn size(&self) -> Result<Size, Error>;
    fn queue_hide_cursor(&mut self) -> Result<(), Error>;
}

pub type RowOrColumnNumber = u16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Size {
    pub height: RowOrColumnNumber,
    pub width: RowOrColumnNumber,
}
