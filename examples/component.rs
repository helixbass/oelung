use crossterm::event::{self, Event, KeyCode};

use oelung::{soft, Component, ComponentInterface, Grid, Renderer, RendererBuilder};

fn main() -> Result<(), anyhow::Error> {
    let mut renderer = RendererBuilder::default().build()?;

    let current_percent = 14;

    render_screen(&mut renderer, current_percent)?;

    loop {
        match event::read()? {
            Event::Key(key_event) if key_event.code == KeyCode::Char('q') => break,
            _ => {}
        }
    }

    Ok(())
}

fn render_screen(renderer: &mut Renderer, current_percent: u32) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %FlexColumn
            children => [
              %Text
                text => "Top area"
                cursor => %Cursor.Relative
                  x => 0
                  y => 0
            ]
            flex_grow => 1
          %StatusBar::new(current_percent)
          %Text "This looks great. Hit q to quit"
        ]
    })?;

    Ok(())
}

struct StatusBar {
    pub current_percent: u32,
}

impl StatusBar {
    pub fn new(current_percent: u32) -> Self {
        Self { current_percent }
    }
}

impl ComponentInterface for StatusBar {
    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        Ok(soft! {
          %Text
            children => [
              %Text "some_file.rs ["
              %Text self.current_percent
              %Text "%] 999 lines |1"
            ]
        })
    }

    fn height(&self) -> Option<u16> {
        Some(1)
    }
}
