use std::io::stdout;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

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
