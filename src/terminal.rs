use std::io::{stdout, StdoutLock};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tracing::instrument;

use crate::Error;

#[instrument(level = "trace")]
pub fn take_over_screen() -> Result<TakeOverScreenGuard, Error> {
    enable_raw_mode().map_err(|_| Error::Crossterm("enable raw mode failed".into()))?;
    let mut stdout = stdout().lock();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|_| Error::Crossterm("enter alternate screen failed".into()))?;

    Ok(TakeOverScreenGuard::new(stdout))
}

pub struct TakeOverScreenGuard {
    pub stdout: StdoutLock<'static>,
}

impl TakeOverScreenGuard {
    pub fn new(stdout: StdoutLock<'static>) -> Self {
        Self { stdout }
    }
}

impl Drop for TakeOverScreenGuard {
    fn drop(&mut self) {
        let Ok(_) = execute!(self.stdout, LeaveAlternateScreen) else {
            return;
        };
        let _ = disable_raw_mode();
    }
}
