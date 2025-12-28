use std::io::{stdout, StdoutLock, Write};
use std::iter;

use crossterm::{
    cursor,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use squalid::_d;

use crate::{
    size, take_over_screen, Component, ComponentInterface, Cursor, Error, Offset, Size, Style,
    TakeOverScreenGuard, Text, TextChild,
};

pub struct Renderer {
    pub take_over_screen_guard: TakeOverScreenGuard,
    pub stdout: StdoutLock<'static>,
    pub size: Size,
    pub rendered_cursor_position_in_this_render: Option<Position>,
    pub staged_this_render: Option<Staged>,
}

impl Renderer {
    pub fn try_new() -> Result<Self, Error> {
        Ok(Self {
            take_over_screen_guard: take_over_screen()?,
            stdout: stdout().lock(),
            size: size()?,
            rendered_cursor_position_in_this_render: _d(),
            staged_this_render: _d(),
        })
    }

    pub fn render<'a>(&mut self, component: Component<'a>) -> Result<(), Error> {
        self.rendered_cursor_position_in_this_render = _d();
        self.size = size()?;

        self.stdout
            .queue(Clear(ClearType::All))
            .map_err(|_| Error::Crossterm("clear failed".into()))?;
        self.stdout
            .queue(cursor::Hide)
            .map_err(|_| Error::Crossterm("hide failed".into()))?;
        self.stdout
            .queue(cursor::MoveTo(0, 0))
            .map_err(|_| Error::Crossterm("move to failed".into()))?;

        let style = Style::default();
        let grid = Grid {
            left: 0,
            top: 0,
            width: self.size.width,
            height: self.size.height,
        };
        let mut component = component;
        let mut outer_components: Vec<Component<'a>> = _d();
        while matches!(component, Component::Component(_)) {
            outer_components.push(component);
            component = outer_components[outer_components.len() - 1]
                .as_component()
                .render(grid)?;
        }
        let mut rendering_context = RenderingContext::new(grid, style);
        rendering_context.render(component)?;
        let RenderingContext {
            staged,
            rendered_cursor_position,
            ..
        } = rendering_context;
        self.staged_this_render = Some(staged);
        if let Some(rendered_cursor_position) = rendered_cursor_position {
            if self.rendered_cursor_position_in_this_render.is_some() {
                return Err(Error::RenderedCursorMoreThanOnce);
            }
            self.rendered_cursor_position_in_this_render = Some(rendered_cursor_position);
        }

        self.render_staged()?;

        if let Some(cursor_position) = self.rendered_cursor_position_in_this_render {
            self.stdout
                .queue(cursor::MoveTo(cursor_position.column, cursor_position.row))
                .map_err(|_| Error::Crossterm("move to failed".into()))?;
            self.stdout
                .queue(cursor::Show)
                .map_err(|_| Error::Crossterm("show failed".into()))?;
        }

        self.stdout
            .flush()
            .map_err(|_| Error::Crossterm("flush failed".into()))?;

        Ok(())
    }

    fn render_staged(&mut self) -> Result<(), Error> {
        let staged = self.staged_this_render.as_ref().unwrap();
        for (row_index, row) in staged.lines.iter().enumerate() {
            for (styled_chunk, style) in row {
                match style.color {
                    Some(color) => {
                        self.stdout
                            .queue(SetForegroundColor(color))
                            .map_err(|_| Error::Crossterm("set foreground color failed".into()))?;
                    }
                    None => {
                        self.stdout
                            .queue(SetForegroundColor(Color::Reset))
                            .map_err(|_| Error::Crossterm("set foreground color failed".into()))?;
                    }
                }
                self.stdout
                    .queue(Print(styled_chunk))
                    .map_err(|_| Error::Crossterm("print failed".into()))?;
            }

            if row_index < staged.lines.len() - 1 {
                self.stdout
                    .queue(Print("\r\n"))
                    .map_err(|_| Error::Crossterm("print failed".into()))?;
            }
        }

        Ok(())
    }
}

pub struct RenderingContext {
    pub grid: Grid,
    pub staged: Staged,
    pub rendered_cursor_position: Option<Position>,
    // pub current_line_number: Option<usize>,
    pub style: Style,
}

impl RenderingContext {
    pub fn new(grid: Grid, style: Style) -> Self {
        Self {
            grid,
            staged: _d(),
            rendered_cursor_position: _d(),
            // current_line_number: _d(),
            style,
        }
    }

    pub fn render(&mut self, component: Component) -> Result<(), Error> {
        match component {
            Component::Text(text) => {
                self.staged.lines.push(_d());
                self.render_text(text, 0, self.style)?;
            }
            Component::FlexColumn(flex_column) => {
                assert!(
                    flex_column
                        .children
                        .iter()
                        .filter(|child| child.flex_grow() == Some(1.0) && child.height().is_none())
                        .count()
                        <= 1
                );
                assert!(flex_column
                    .children
                    .iter()
                    .all(
                        |child| child.flex_grow() == Some(1.0) && child.height().is_none()
                            || child.flex_grow() == None && child.height() == Some(1)
                    ));
                let mut num_rows_rendered = 0;
                let num_children = flex_column.children.len();
                for mut child in flex_column.children {
                    let height = if child.height() == Some(1) {
                        1
                    } else {
                        self.grid.height - (u16::try_from(num_children).unwrap() - 1)
                    };
                    let grid = Grid {
                        left: self.grid.left,
                        top: self.grid.top + num_rows_rendered,
                        width: self.grid.width,
                        height,
                    };
                    while matches!(child, Component::Component(_)) {
                        child = child.into_component().render(grid)?;
                    }
                    let mut rendering_context = RenderingContext::new(grid, self.style);
                    rendering_context.render(child)?;
                    let RenderingContext {
                        staged,
                        rendered_cursor_position,
                        ..
                    } = rendering_context;
                    assert!(staged.lines.len() <= usize::from(height));
                    let num_less_rendered_vs_height = usize::from(height) - staged.lines.len();
                    self.staged.lines.extend(staged.lines);
                    if num_less_rendered_vs_height > 0 {
                        self.staged
                            .lines
                            .extend(iter::repeat(vec![]).take(num_less_rendered_vs_height));
                    }
                    num_rows_rendered += height;
                    if let Some(rendered_cursor_position) = rendered_cursor_position {
                        if self.rendered_cursor_position.is_some() {
                            return Err(Error::RenderedCursorMoreThanOnce);
                        }
                        self.rendered_cursor_position = Some(rendered_cursor_position);
                    }
                }
                if let Some(cursor) = flex_column.cursor {
                    self.render_cursor(cursor)?;
                }
            }
            Component::Component(_) => unreachable!(),
        }

        Ok(())
    }

    pub fn render_text(
        &mut self,
        text: Text,
        line_num: usize,
        mut style: Style,
    ) -> Result<(), Error> {
        if let Some(text_style) = text.style {
            if let Some(color) = text_style.color {
                style.color = Some(color);
            }
        }

        for child in text.children {
            match child {
                TextChild::Text(text) => self.print_text(&text, line_num, style)?,
                TextChild::Nested(text) => self.render_text(*text, line_num, style)?,
                TextChild::NestedComponent(component) => {
                    let mut rendered = component.render(self.grid)?;
                    while matches!(rendered, Component::Component(_)) {
                        rendered = rendered.into_component().render(self.grid)?;
                    }
                    let rendered = match rendered {
                        Component::Text(rendered) => rendered,
                        _ => return Err(Error::RenderedNonTextChildInText),
                    };
                    self.render_text(rendered, line_num, style)?;
                }
            }
        }

        if let Some(cursor) = text.cursor {
            self.render_cursor(cursor)?;
        }

        Ok(())
    }

    fn print_text(&mut self, text: &str, line_num: usize, style: Style) -> Result<(), Error> {
        self.staged.lines[line_num].push((text.to_owned(), style));

        Ok(())
    }

    fn render_cursor(&mut self, cursor: Cursor) -> Result<(), Error> {
        if self.rendered_cursor_position.is_some() {
            return Err(Error::RenderedCursorMoreThanOnce);
        }
        self.rendered_cursor_position = Some(match cursor {
            Cursor::Relative(Offset { x, y }) => Position { row: y, column: x },
        });

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub row: u16,
    pub column: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    pub left: u16,
    pub top: u16,
    pub height: u16,
    pub width: u16,
}

#[derive(Default)]
pub struct Staged {
    pub lines: Vec<Vec<(String, Style)>>,
}
