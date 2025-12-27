use std::io::{stdout, StdoutLock, Write};

use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    QueueableCommand,
};

use crate::{size, take_over_screen, Component, Error, Size, TakeOverScreenGuard, Text, TextChild};

pub struct Renderer {
    pub take_over_screen_guard: TakeOverScreenGuard,
    pub stdout: StdoutLock<'static>,
    pub size: Size,
}

impl Renderer {
    pub fn try_new() -> Result<Self, Error> {
        Ok(Self {
            take_over_screen_guard: take_over_screen()?,
            stdout: stdout().lock(),
            size: size()?,
        })
    }

    pub fn render(&mut self, component: Component) -> Result<(), Error> {
        self.size = size()?;

        self.stdout
            .queue(Clear(ClearType::All))
            .map_err(|_| Error::Crossterm("clear failed".into()))?;
        self.stdout
            .queue(cursor::SavePosition)
            .map_err(|_| Error::Crossterm("save position failed".into()))?;
        self.stdout
            .queue(cursor::Hide)
            .map_err(|_| Error::Crossterm("hide failed".into()))?;
        self.stdout
            .queue(cursor::MoveTo(0, 0))
            .map_err(|_| Error::Crossterm("move to failed".into()))?;

        match component {
            Component::Text(text) => self.render_text(text)?,
        }

        self.stdout
            .queue(cursor::RestorePosition)
            .map_err(|_| Error::Crossterm("restore position failed".into()))?;
        self.stdout
            .queue(cursor::Show)
            .map_err(|_| Error::Crossterm("show failed".into()))?;

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
            }
        }

        Ok(())
    }

    fn print_text(&mut self, text: &str) -> Result<(), Error> {
        unimplemented!()
    }
}
