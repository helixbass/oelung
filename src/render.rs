use std::cell::UnsafeCell;
use std::io::Write;
use std::iter;
use std::mem;

use crossterm::{
    cursor,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use smol_str::{SmolStr, ToSmolStr};
use squalid::_d;
use tracing::instrument;

use crate::{
    size, take_over_screen, Component, ComponentInterface, Cursor, Error, Offset, Size, Style,
    TakeOverScreenGuard, Text, TextChild,
};

pub struct Renderer {
    pub take_over_screen_guard: TakeOverScreenGuard,
    pub size: Size,
    pub rendered_cursor_position_in_this_render: Option<Position>,
    pub grids: [Staged; 2],
    pub last_rendered_grid_index: Option<usize>,
}

impl Renderer {
    #[instrument(level = "trace")]
    pub fn try_new() -> Result<Self, Error> {
        let size = size()?;
        let default_grid_row = StyledChunk::new(" ".repeat(usize::from(size.width)).into(), _d());
        let mut take_over_screen_guard = take_over_screen()?;

        take_over_screen_guard
            .stdout
            .execute(Clear(ClearType::All))
            .map_err(|_| Error::Crossterm("clear failed".into()))?;

        Ok(Self {
            take_over_screen_guard,
            size,
            rendered_cursor_position_in_this_render: _d(),
            last_rendered_grid_index: _d(),
            grids: [
                vec![vec![default_grid_row.clone()]; usize::from(size.height)],
                vec![vec![default_grid_row]; usize::from(size.height)],
            ],
        })
    }

    #[instrument(level = "trace", skip(self, component))]
    pub fn render(&mut self, component: Component) -> Result<(), Error> {
        self.rendered_cursor_position_in_this_render = _d();

        self.take_over_screen_guard
            .stdout
            .queue(cursor::Hide)
            .map_err(|_| Error::Crossterm("hide failed".into()))?;

        let style = Style::default();
        let grid = Grid {
            left: 0,
            top: 0,
            width: self.size.width,
            height: self.size.height,
        };

        let component_holder = ComponentHolder::default();
        component_holder.push(component);
        while matches!(component_holder.get_most_recent(), Component::Component(_)) {
            component_holder.render_most_recent_and_push(grid)?;
        }

        let mut rendering_context = RenderingContext::new(grid, style);
        rendering_context.render(component_holder.get_most_recent())?;
        let RenderingContext {
            staged,
            rendered_cursor_position,
            ..
        } = rendering_context;
        self.grids[match self.last_rendered_grid_index {
            None => 0,
            Some(0) => 1,
            Some(1) => 0,
            _ => unreachable!(),
        }] = staged;
        if let Some(rendered_cursor_position) = rendered_cursor_position {
            // TODO: looks like this could never be true?
            if self.rendered_cursor_position_in_this_render.is_some() {
                return Err(Error::RenderedCursorMoreThanOnce);
            }
            self.rendered_cursor_position_in_this_render = Some(rendered_cursor_position);
        }

        self.render_staged()?;

        if let Some(cursor_position) = self.rendered_cursor_position_in_this_render {
            self.take_over_screen_guard
                .stdout
                .queue(cursor::MoveTo(cursor_position.column, cursor_position.row))
                .map_err(|_| Error::Crossterm("move to failed".into()))?;
            self.take_over_screen_guard
                .stdout
                .queue(cursor::Show)
                .map_err(|_| Error::Crossterm("show failed".into()))?;
        }

        self.take_over_screen_guard
            .stdout
            .flush()
            .map_err(|_| Error::Crossterm("flush failed".into()))?;

        self.last_rendered_grid_index = Some(match self.last_rendered_grid_index {
            None => 0,
            Some(0) => 1,
            Some(1) => 0,
            _ => unreachable!(),
        });

        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    fn render_staged(&mut self) -> Result<(), Error> {
        let staged = &self.grids[match self.last_rendered_grid_index {
            None => 0,
            Some(0) => 1,
            Some(1) => 0,
            _ => unreachable!(),
        }];
        let prev_staged = self
            .last_rendered_grid_index
            .map(|last_rendered_grid_index| &self.grids[last_rendered_grid_index]);
        for (row_index, row) in staged.iter().enumerate() {
            let prev_staged_row = prev_staged.map(|prev_staged| &prev_staged[row_index]);
            enum MatchesPrevStagedRow {
                MatchesPrefixNumChunksAndLength(usize, u16),
                MatchesWholeRow,
            }
            let matches_prev_staged_row: Option<MatchesPrevStagedRow> =
                prev_staged_row.and_then(|prev_staged_row| {
                    let mut num_matched_chunks_and_length: (usize, u16) = (0, 0);
                    for (index, styled_chunk) in row.into_iter().enumerate() {
                        if prev_staged_row.get(index) != Some(styled_chunk) {
                            return match num_matched_chunks_and_length.0 {
                                0 => None,
                                _ => Some(MatchesPrevStagedRow::MatchesPrefixNumChunksAndLength(
                                    num_matched_chunks_and_length.0,
                                    num_matched_chunks_and_length.1,
                                )),
                            };
                        }
                        num_matched_chunks_and_length = (
                            num_matched_chunks_and_length.0 + 1,
                            num_matched_chunks_and_length.1
                                + u16::try_from(styled_chunk.str.len()).unwrap(),
                        );
                    }
                    Some(MatchesPrevStagedRow::MatchesWholeRow)
                });
            if matches!(
                matches_prev_staged_row,
                Some(MatchesPrevStagedRow::MatchesWholeRow)
            ) {
                continue;
            }

            self.take_over_screen_guard
                .stdout
                .queue(cursor::MoveTo(
                    match matches_prev_staged_row {
                        None => 0,
                        Some(MatchesPrevStagedRow::MatchesPrefixNumChunksAndLength(_, len)) => len,
                        _ => unreachable!(),
                    },
                    u16::try_from(row_index).unwrap(),
                ))
                .map_err(|_| Error::Crossterm("move to failed".into()))?;

            for styled_chunk in row.into_iter().skip(match matches_prev_staged_row {
                None => 0,
                Some(MatchesPrevStagedRow::MatchesPrefixNumChunksAndLength(num_chunks, _)) => {
                    num_chunks
                }
                _ => unreachable!(),
            }) {
                match styled_chunk.style.color {
                    Some(color) => {
                        self.take_over_screen_guard
                            .stdout
                            .queue(SetForegroundColor(color))
                            .map_err(|_| Error::Crossterm("set foreground color failed".into()))?;
                    }
                    None => {
                        self.take_over_screen_guard
                            .stdout
                            .queue(SetForegroundColor(Color::Reset))
                            .map_err(|_| Error::Crossterm("set foreground color failed".into()))?;
                    }
                }
                match styled_chunk.style.background_color {
                    Some(background_color) => {
                        self.take_over_screen_guard
                            .stdout
                            .queue(SetBackgroundColor(background_color))
                            .map_err(|_| Error::Crossterm("set background color failed".into()))?;
                    }
                    None => {
                        self.take_over_screen_guard
                            .stdout
                            .queue(SetBackgroundColor(Color::Reset))
                            .map_err(|_| Error::Crossterm("set background color failed".into()))?;
                    }
                }
                self.take_over_screen_guard
                    .stdout
                    .queue(Print(styled_chunk.str.clone()))
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

    #[instrument(level = "trace", skip(self, component))]
    pub fn render(&mut self, component: &Component) -> Result<(), Error> {
        match component {
            Component::Text(text) => {
                self.staged.push(_d());
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
                for child in &flex_column.children {
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
                    let mut rendering_context = RenderingContext::new(grid, self.style);

                    if matches!(child, Component::Component(_)) {
                        let component_holder = ComponentHolder::default();
                        component_holder.push(child.as_component().render(grid)?);
                        while matches!(component_holder.get_most_recent(), Component::Component(_))
                        {
                            component_holder.render_most_recent_and_push(grid)?;
                        }
                        rendering_context.render(component_holder.get_most_recent())?;
                    } else {
                        rendering_context.render(child)?;
                    };
                    let RenderingContext {
                        staged,
                        rendered_cursor_position,
                        ..
                    } = rendering_context;
                    assert!(staged.len() <= usize::from(height));
                    let num_less_rendered_vs_height = usize::from(height) - staged.len();
                    self.staged.extend(staged);
                    if num_less_rendered_vs_height > 0 {
                        self.staged
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

    // #[instrument(level = "trace", skip(self, text, line_num, style))]
    pub fn render_text(
        &mut self,
        text: &Text,
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

        for child in &text.children {
            match child {
                TextChild::Text(text) => self.print_text(text, line_num, style)?,
                TextChild::Nested(text) => self.render_text(text, line_num, style)?,
                TextChild::NestedComponent(component) => {
                    let rendered = component.render(self.grid)?;
                    if matches!(&rendered, Component::Component(_)) {
                        let component_holder = ComponentHolder::default();
                        component_holder.push(rendered);
                        while matches!(component_holder.get_most_recent(), Component::Component(_))
                        {
                            component_holder.render_most_recent_and_push(self.grid)?;
                        }
                        let rendered = match component_holder.get_most_recent() {
                            Component::Text(rendered) => rendered,
                            _ => return Err(Error::RenderedNonTextChildInText),
                        };
                        self.render_text(rendered, line_num, style)?;
                    } else {
                        let rendered = match rendered {
                            Component::Text(rendered) => rendered,
                            _ => return Err(Error::RenderedNonTextChildInText),
                        };
                        self.render_text(&rendered, line_num, style)?;
                    }
                }
            }
        }

        if let Some(cursor) = text.cursor {
            self.render_cursor(cursor)?;
        }

        Ok(())
    }

    // #[instrument(level = "trace", skip(self, text, line_num, style))]
    fn print_text(&mut self, text: &str, line_num: usize, style: Style) -> Result<(), Error> {
        self.staged[line_num].push(StyledChunk {
            str: text.to_smolstr(),
            style,
        });

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

pub type Staged = Vec<Vec<StyledChunk>>;

#[derive(Clone, PartialEq, Eq)]
pub struct StyledChunk {
    pub str: SmolStr,
    pub style: Style,
}

impl StyledChunk {
    pub fn new(str: SmolStr, style: Style) -> Self {
        Self { str, style }
    }
}

#[derive(Default)]
struct ComponentHolder<'a> {
    store: Vec<Vec<Component<'a>>>,
    len: usize,
}

impl<'a> ComponentHolder<'a> {
    pub const PER_ROW: usize = 20;

    pub fn push(&self, component: Component<'a>) {
        let store_index = self.len / Self::PER_ROW;
        let self_ptr = self as *const Self as *mut Self;
        // SAFETY: this should be fine because we're making sure
        // that nothing in the store moves after it's been added,
        // and the only thing that holds references to us is
        // `.get_most_recent()`
        unsafe {
            let self_: &mut Self = &mut *self_ptr;
            if self_.len % Self::PER_ROW == 0 {
                self_.store.push(Vec::with_capacity(Self::PER_ROW));
            }
            self_.store[store_index].push(component);
            self_.len += 1;
        }
    }

    pub fn get_most_recent(&self) -> &Component<'a> {
        assert!(self.len > 0);
        let store_index = (self.len - 1) / Self::PER_ROW;
        let index_in_row = (self.len - 1) % Self::PER_ROW;
        &self.store[store_index][index_in_row]
    }

    pub fn render_most_recent_and_push(&self, grid: Grid) -> Result<(), Error> {
        let most_recent = self.get_most_recent();
        let rendered = most_recent.as_component().render(grid)?;
        self.push(
            // SAFETY: I think again here this is fine because of
            // the way we're promising to use/drop ComponentHolder,
            // I think it's angry because really what's being rendered
            // here is a component whose lifetime is attached to `self`,
            // not `'a`
            unsafe { mem::transmute::<_, Component<'a>>(rendered) },
        );

        Ok(())
    }
}
