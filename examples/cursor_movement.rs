use crossterm::event::{self, Event, KeyCode};

use oelung::{soft, Component, ComponentInterface, FlexColumnBuilder, Grid, Renderer};

fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let lines = vec![
        "Hello world".to_owned(),
        "This all looks phenomenal".to_owned(),
        "Can't say enough good things about it".to_owned(),
        "Well maybe it's terrible in fact".to_owned(),
        "I don't really know do I".to_owned(),
    ];

    let mut cursor_position = Position { row: 0, column: 0 };

    let mut current_percent = cursor_position
        .row
        .div_ceil(u16::try_from(lines.len()).unwrap());

    render_screen(&mut renderer, current_percent, cursor_position)?;

    loop {
        match event::read()? {
            Event::Key(key_event) if key_event.code == KeyCode::Char('q') => break,
            _ => {}
        }
    }

    Ok(())
}

fn render_screen(
    renderer: &mut Renderer,
    current_percent: u16,
    cursor_position: Position,
    lines: &[String],
) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %TextArea::new(lines)
          %StatusBar::new(current_percent)
          %Text "Hit q to quit. Use j/k/h/l to move around the text area."
        ]
    })?;

    Ok(())
}

struct TextArea<'a> {
    pub lines: &'a [String],
}

impl<'a> TextArea<'a> {
    pub fn new(lines: &'a [String]) -> Self {
        Self { lines }
    }
}

impl<'a> ComponentInterface for TextArea<'a> {
    fn render(&self, _grid: Grid) -> Result<Component, anyhow::Error> {
        let mut flex_column = FlexColumnBuilder::default();
        Ok(soft! {
          %FlexColumn
            children => [
              %Text
                text => "Top area"
                cursor => %Cursor.Relative
                  x => cursor_position.x
                  y => cursor_position.y
            ]
            flex_grow => 1
        })
    }
}

struct StatusBar {
    pub current_percent: u16,
}

impl StatusBar {
    pub fn new(current_percent: u16) -> Self {
        Self { current_percent }
    }
}

impl ComponentInterface for StatusBar {
    fn render(&self, _grid: Grid) -> Result<Component, anyhow::Error> {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Position {
    row: u16,
    column: u16,
}
