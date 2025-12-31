use std::cell::Cell;

use crossterm::event::{self, Event, KeyCode};

use oelung::{soft, Component, ComponentInterface, Cursor, FlexColumnBuilder, Grid, Renderer};
use squalid::_d;

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

    let last_rendered_text_area_grid: Cell<Option<Grid>> = _d();

    render_screen(
        &mut renderer,
        cursor_position,
        &lines,
        &last_rendered_text_area_grid,
    )?;

    loop {
        match event::read()? {
            Event::Key(key_event) if key_event.code == KeyCode::Char('q') => break,
            Event::Key(key_event) if key_event.code == KeyCode::Char('j') => {
                if cursor_position.row >= u16::try_from(lines.len()).unwrap() - 1 {
                    continue;
                }
                cursor_position.row += 1;
                render_screen(
                    &mut renderer,
                    cursor_position,
                    &lines,
                    &last_rendered_text_area_grid,
                )?;
            }
            Event::Key(key_event) if key_event.code == KeyCode::Char('k') => {
                if cursor_position.row == 0 {
                    continue;
                }
                cursor_position.row -= 1;
                render_screen(
                    &mut renderer,
                    cursor_position,
                    &lines,
                    &last_rendered_text_area_grid,
                )?;
            }
            Event::Key(key_event) if key_event.code == KeyCode::Char('l') => {
                if cursor_position.column == last_rendered_text_area_grid.get().unwrap().width - 1 {
                    continue;
                }
                cursor_position.column += 1;
                render_screen(
                    &mut renderer,
                    cursor_position,
                    &lines,
                    &last_rendered_text_area_grid,
                )?;
            }
            Event::Key(key_event) if key_event.code == KeyCode::Char('h') => {
                if cursor_position.column == 0 {
                    continue;
                }
                cursor_position.column -= 1;
                render_screen(
                    &mut renderer,
                    cursor_position,
                    &lines,
                    &last_rendered_text_area_grid,
                )?;
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
    last_rendered_text_area_grid: &Cell<Option<Grid>>,
) -> Result<(), anyhow::Error> {
    let current_percent = ((f64::from(cursor_position.row)
        / f64::from(u32::try_from(lines.len()).unwrap()))
        * 100.0) as u16;

    // renderer.render(soft! {
    //   %FlexColumn
    //     children => [
    //       %TextArea::new(lines, cursor_position, last_rendered_text_area_grid)
    //       %StatusBar::new(current_percent, cursor_position.column)
    //       %Text "Hit q to quit. Use j/k/h/l to move around the text area."
    //     ]
    // })?;
    let flex_column = oelung::FlexColumn {
        children: vec![Component::Component(Box::new(TextArea::new(
            lines,
            cursor_position,
            last_rendered_text_area_grid,
        )))],
        flex_grow: None,
        cursor: None,
    };
    renderer.render(Component::FlexColumn(flex_column));

    Ok(())
}

struct TextArea<'a> {
    pub lines: &'a [String],
    pub cursor_position: Position,
    last_rendered_text_area_grid: &'a Cell<Option<Grid>>,
}

impl<'a> TextArea<'a> {
    pub fn new(
        lines: &'a [String],
        cursor_position: Position,
        last_rendered_text_area_grid: &'a Cell<Option<Grid>>,
    ) -> Self {
        Self {
            lines,
            cursor_position,
            last_rendered_text_area_grid,
        }
    }
}

impl<'a> ComponentInterface<'a> for TextArea<'a> {
    fn render(&self, grid: Grid) -> Result<Component<'a>, anyhow::Error> {
        self.last_rendered_text_area_grid.set(Some(grid));
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
    pub cursor_column: u16,
}

impl StatusBar {
    pub fn new(current_percent: u16, cursor_column: u16) -> Self {
        Self {
            current_percent,
            cursor_column,
        }
    }
}

impl ComponentInterface<'static> for StatusBar {
    fn render<'a>(&self, _grid: Grid) -> Result<Component<'static>, anyhow::Error> {
        Ok(soft! {
          %Text
            color => Ansi(182)
            children => [
              %Text "some_file.rs ["
              %Text self.current_percent
              %Text "%] 999 lines |"
              %Text self.cursor_column
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
