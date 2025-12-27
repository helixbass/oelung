use std::io::stdout;

use crossterm::{
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use squalid::EverythingExt;

use crate::Error;

pub fn take_over_screen() -> Result<TakeOverScreenGuard, Error> {
    enable_raw_mode().map_err(|_| Error::Crossterm("enable raw mode failed".into()))?;
    execute!(stdout(), EnterAlternateScreen)
        .map_err(|_| Error::Crossterm("enter alternate screen failed".into()))?;

    Ok(TakeOverScreenGuard::default())
}

#[derive(Default)]
pub struct TakeOverScreenGuard {}

impl Drop for TakeOverScreenGuard {
    fn drop(&mut self) {
        let Ok(_) = execute!(stdout(), LeaveAlternateScreen) else {
            return;
        };
        let _ = disable_raw_mode();
    }
}

pub fn size() -> Result<Size, Error> {
    Ok(terminal::size()
        .map_err(|_| Error::Crossterm("size failed".into()))?
        .thrush(|size| Size {
            height: size.1,
            width: size.0,
        }))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Size {
    pub height: u16,
    pub width: u16,
}
