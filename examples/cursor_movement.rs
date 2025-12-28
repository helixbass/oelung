use crossterm::event::{self, Event, KeyCode};

use oelung::{soft, Component, ComponentInterface, Cursor, FlexColumnBuilder, Grid, Renderer};

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

    render_screen(&mut renderer, cursor_position, &lines)?;

    loop {
        match event::read()? {
            Event::Key(key_event) if key_event.code == KeyCode::Char('q') => break,
            Event::Key(key_event) if key_event.code == KeyCode::Char('j') => {
                cursor_position.row += 1;
                render_screen(&mut renderer, cursor_position, &lines)?;
            }
            Event::Key(key_event) if key_event.code == KeyCode::Char('k') => {
                cursor_position.row -= 1;
                render_screen(&mut renderer, cursor_position, &lines)?;
            }
            Event::Key(key_event) if key_event.code == KeyCode::Char('l') => {
                cursor_position.column += 1;
                render_screen(&mut renderer, cursor_position, &lines)?;
            }
            Event::Key(key_event) if key_event.code == KeyCode::Char('h') => {
                cursor_position.column -= 1;
                render_screen(&mut renderer, cursor_position, &lines)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn render_screen(
    renderer: &mut Renderer,
    cursor_position: Position,
    lines: &[String],
) -> Result<(), anyhow::Error> {
    let current_percent = cursor_position
        .row
        .div_ceil(u16::try_from(lines.len()).unwrap());

    renderer.render(soft! {
      %FlexColumn
        children => [
          %TextArea::new(lines, cursor_position)
          %StatusBar::new(current_percent)
          %Text "Hit q to quit. Use j/k/h/l to move around the text area."
        ]
    })?;

    Ok(())
}

struct TextArea<'a> {
    pub lines: &'a [String],
    pub cursor_position: Position,
}

impl<'a> TextArea<'a> {
    pub fn new(lines: &'a [String], cursor_position: Position) -> Self {
        Self {
            lines,
            cursor_position,
        }
    }
}

impl<'a> ComponentInterface for TextArea<'a> {
    fn render<'b>(&self, _grid: Grid) -> Result<Component<'b>, anyhow::Error> {
        let mut flex_column = FlexColumnBuilder::default();
        for line in self.lines {
            flex_column = flex_column.child(soft! {
                %Text line
            });
        }
        flex_column = flex_column.flex_grow(1);
        flex_column = flex_column.cursor(
            Cursor::relative()
                .x(self.cursor_position.column)
                .y(self.cursor_position.row),
        );
        Ok(flex_column.build()?.into())
    }

    fn flex_grow(&self) -> Option<f64> {
        Some(1.0)
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
    fn render<'a>(&self, _grid: Grid) -> Result<Component<'a>, anyhow::Error> {
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
