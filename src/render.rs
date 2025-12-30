use std::io::{stdout, StdoutLock, Write};
use std::iter;

use crossterm::{
    cursor,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use squalid::_d;
use tracing::{instrument, trace_span};

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
    #[instrument(level = "trace")]
    pub fn try_new() -> Result<Self, Error> {
        Ok(Self {
            take_over_screen_guard: take_over_screen()?,
            stdout: stdout().lock(),
            size: size()?,
            rendered_cursor_position_in_this_render: _d(),
            staged_this_render: _d(),
        })
    }

    #[instrument(level = "trace", skip(self, component))]
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

    #[instrument(level = "trace", skip(self))]
    fn render_staged(&mut self) -> Result<(), Error> {
        let staged = self.staged_this_render.as_ref().unwrap();
        // let buffer: Vec<u8> = Vec::with_capacity(self.size.height * self.size.width);
        let mut buffer: Vec<u8> = _d();
        for (row_index, row) in staged.lines.iter().enumerate() {
            let row_guard = trace_span!("row").entered();
            for (styled_chunk, style) in row {
                let chunk_guard = trace_span!("queueing chunk").entered();
                match style.color {
                    Some(color) => {
                        buffer
                            .queue(SetForegroundColor(color))
                            .map_err(|_| Error::Crossterm("set foreground color failed".into()))?;
                    }
                    None => {
                        buffer
                            .queue(SetForegroundColor(Color::Reset))
                            .map_err(|_| Error::Crossterm("set foreground color failed".into()))?;
                    }
                }
                match style.background_color {
                    Some(background_color) => {
                        buffer
                            .queue(SetBackgroundColor(background_color))
                            .map_err(|_| Error::Crossterm("set background color failed".into()))?;
                    }
                    None => {
                        buffer
                            .queue(SetBackgroundColor(Color::Reset))
                            .map_err(|_| Error::Crossterm("set background color failed".into()))?;
                    }
                }
                buffer
                    .queue(Print(styled_chunk))
                    .map_err(|_| Error::Crossterm("print failed".into()))?;
                drop(chunk_guard);
            }

            if row_index < staged.lines.len() - 1 {
                buffer
                    .queue(Print("\r\n"))
                    .map_err(|_| Error::Crossterm("print failed".into()))?;
            }
            drop(row_guard);
        }
        self.stdout
            .write_all(&buffer)
            .map_err(|_| Error::Crossterm("write failed".into()))?;

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

    #[instrument(level = "trace", skip(self, component))]
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
                            || child.flex_grow() == None && child.height().is_some()
                    ));
                let fixed_children_total_height =
                    flex_column
                        .children
                        .iter()
                        .fold(0, |accum, child| match child.height() {
                            None => accum,
                            Some(height) => accum + height,
                        });
                let mut num_rows_rendered = 0;
                for mut child in flex_column.children {
                    let height = if let Some(height) = child.height() {
                        height
                    } else {
                        self.grid.height - fixed_children_total_height
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

    #[instrument(level = "trace", skip(self, text, line_num, style))]
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
            if let Some(background_color) = text_style.background_color {
                style.background_color = Some(background_color);
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

    #[instrument(level = "trace", skip(self, text, line_num, style))]
    fn print_text(&mut self, text: &str, line_num: usize, style: Style) -> Result<(), Error> {
        self.staged.lines[line_num].push((text.to_owned(), style));

        Ok(())
    }

    #[instrument(level = "trace", skip(self, cursor))]
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
