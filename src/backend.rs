use crossterm::terminal;
use squalid::EverythingExt;
use tracing::instrument;

use crate::Error;

#[derive(Debug)]
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
}

#[derive(Debug, Default)]
pub struct BackendCrossterm {}

impl BackendInterface for BackendCrossterm {
    #[instrument(level = "trace")]
    fn size(&self) -> Result<Size, Error> {
        Ok(terminal::size()
            .map_err(|_| Error::Crossterm("size failed".into()))?
            .thrush(|size| Size {
                height: size.1,
                width: size.0,
            }))
    }
}

#[derive(Debug)]
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
}

pub trait BackendInterface {
    fn size(&self) -> Result<Size, Error>;
}

pub type RowOrColumnNumber = u16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Size {
    pub height: RowOrColumnNumber,
    pub width: RowOrColumnNumber,
}
