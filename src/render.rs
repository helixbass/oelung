use std::cell::{Cell, UnsafeCell};
use std::cmp::Ordering;
use std::io::Write;
use std::iter;
use std::mem;

use crossterm::{
    cursor,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use smallvec::{smallvec, SmallVec};
use smol_str::{SmolStr, ToSmolStr};
use squalid::{EverythingExt, _d};
use tracing::instrument;

use crate::{
    size, take_over_screen, Component, Cursor, Error, Offset, Relative, Size, Style,
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
                smallvec![smallvec![default_grid_row.clone()]; usize::from(size.height)],
                smallvec![smallvec![default_grid_row]; usize::from(size.height)],
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
    pub style: Style,
}

impl RenderingContext {
    pub fn new(grid: Grid, style: Style) -> Self {
        Self {
            grid,
            staged: _d(),
            rendered_cursor_position: _d(),
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
                let (absolute_children, non_absolute_children) = flex_column
                    .children
                    .iter()
                    .partition::<SmallVec<&'_ Component<'_>, 10>, _>(|child| {
                        matches!(child, Component::Absolute(_))
                    })
                    .thrush(|(absolute_children, non_absolute_children)| {
                        (
                            absolute_children
                                .into_iter()
                                .map(|absolute| absolute.as_absolute())
                                .collect::<SmallVec<_, 10>>(),
                            non_absolute_children,
                        )
                    });
                assert!(
                    non_absolute_children
                        .iter()
                        .filter(|child| {
                            child.flex_grow() == Some(1.0) && child.height().is_none()
                        })
                        .count()
                        <= 1
                );
                assert!(non_absolute_children.iter().all(|child| {
                    child.flex_grow() == Some(1.0) && child.height().is_none()
                        || child.flex_grow() == None && child.height().is_some()
                }));
                let fixed_children_total_height =
                    non_absolute_children
                        .iter()
                        .fold(0, |accum, child| match child.height() {
                            None => accum,
                            Some(height) => accum + height,
                        });
                let mut num_rows_rendered = 0;
                for child in &non_absolute_children {
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
                            .extend(iter::repeat(smallvec![]).take(num_less_rendered_vs_height));
                    }
                    num_rows_rendered += height;
                    if let Some(rendered_cursor_position) = rendered_cursor_position {
                        if self.rendered_cursor_position.is_some() {
                            return Err(Error::RenderedCursorMoreThanOnce);
                        }
                        self.rendered_cursor_position = Some(rendered_cursor_position);
                    }
                }
                if num_rows_rendered < self.grid.height {
                    self.staged.extend(
                        iter::repeat(smallvec![])
                            .take(usize::from(self.grid.height - num_rows_rendered)),
                    );
                }
                if !absolute_children.is_empty() {
                    assert_eq!(flex_column.relative, Some(Relative::NotMoved));
                }
                for child in &absolute_children {
                    let grid = self.grid;
                    let mut rendering_context = RenderingContext::new(grid, self.style);
                    let child = &child.content;

                    if matches!(&**child, Component::Component(_)) {
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
                    assert!(staged.len() <= usize::from(grid.height));
                    paint_on_top_of(&mut self.staged, staged);
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
            Component::Absolute(_) => unreachable!(),
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

pub type Staged = SmallVec<SmallVec<StyledChunk, 10>, 10>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyledChunk {
    pub str: SmolStr,
    pub style: Style,
}

impl StyledChunk {
    pub fn new(str: SmolStr, style: Style) -> Self {
        Self { str, style }
    }
}

fn paint_on_top_of(onto: &mut Staged, from: Staged) {
    for (row_index, row) in from.into_iter().enumerate() {
        // let onto_row = &mut onto[row_index];
        let mut new_row: SmallVec<StyledChunk, 10> = _d();
        #[derive(Copy, Clone)]
        enum ProgressInOldRow {
            InProgress(usize),
            FullyPast(usize),
        }
        let mut progress_in_old_row: Option<ProgressInOldRow> = _d();
        let mut num_bytes_fully_past_in_old_row = 0;
        let mut num_bytes_already_seen_in_new_row = 0;
        for chunk in row {
            let chunk_len = chunk.str.len();
            let end_byte_of_new_chunk = num_bytes_already_seen_in_new_row + chunk_len;
            'chunk: {
                match chunk.style.background_color {
                    Some(_) => {
                        new_row.push(chunk);
                        progress_in_old_row = 'outer: {
                            let mut next_old_row_index = match progress_in_old_row {
                                None => 0,
                                Some(ProgressInOldRow::FullyPast(fully_past_old_row_index)) => {
                                    fully_past_old_row_index + 1
                                }
                                Some(ProgressInOldRow::InProgress(in_progress_old_row_index)) => {
                                    let in_progress_old_row_chunk_len =
                                        onto[row_index][in_progress_old_row_index].str.len();
                                    let end_byte_of_in_progress = num_bytes_fully_past_in_old_row
                                        + in_progress_old_row_chunk_len;
                                    if end_byte_of_in_progress > end_byte_of_new_chunk {
                                        break 'chunk;
                                    }
                                    num_bytes_fully_past_in_old_row +=
                                        in_progress_old_row_chunk_len;
                                    in_progress_old_row_index + 1
                                }
                            };
                            loop {
                                let Some(next_old_chunk) = onto[row_index].get(next_old_row_index)
                                else {
                                    break 'outer match next_old_row_index {
                                        0 => None,
                                        next_old_row_index => Some(ProgressInOldRow::FullyPast(
                                            next_old_row_index - 1,
                                        )),
                                    };
                                };
                                let end_byte_of_next_old_row_chunk =
                                    num_bytes_fully_past_in_old_row + next_old_chunk.str.len();
                                match end_byte_of_next_old_row_chunk.cmp(&end_byte_of_new_chunk) {
                                    Ordering::Less => {
                                        num_bytes_fully_past_in_old_row += next_old_chunk.str.len();
                                        next_old_row_index += 1;
                                    }
                                    Ordering::Equal => {
                                        num_bytes_fully_past_in_old_row += next_old_chunk.str.len();
                                        break 'outer Some(ProgressInOldRow::FullyPast(
                                            next_old_row_index,
                                        ));
                                    }
                                    Ordering::Greater => {
                                        break 'outer Some(ProgressInOldRow::InProgress(
                                            next_old_row_index,
                                        ));
                                    }
                                }
                            }
                        };
                    }
                    None => {
                        progress_in_old_row = 'outer: {
                            let mut num_bytes_already_printed_in_this_new_chunk = 0;
                            let mut next_old_row_index = match progress_in_old_row {
                                None => 0,
                                Some(ProgressInOldRow::FullyPast(fully_past_old_row_index)) => {
                                    fully_past_old_row_index + 1
                                }
                                Some(ProgressInOldRow::InProgress(in_progress_old_row_index)) => {
                                    let in_progress_old_row_chunk =
                                        &onto[row_index][in_progress_old_row_index];
                                    let in_progress_old_row_chunk_len =
                                        in_progress_old_row_chunk.str.len();
                                    let end_byte_of_in_progress = num_bytes_fully_past_in_old_row
                                        + in_progress_old_row_chunk_len;
                                    if end_byte_of_in_progress > end_byte_of_new_chunk {
                                        new_row.push(StyledChunk {
                                            str: chunk.str,
                                            style: Style {
                                                color: chunk.style.color,
                                                background_color: in_progress_old_row_chunk
                                                    .style
                                                    .background_color,
                                            },
                                        });
                                        break 'chunk;
                                    }
                                    let num_remaining_bytes_in_in_progress_to_print =
                                        end_byte_of_in_progress - end_byte_of_new_chunk;
                                    new_row.push(StyledChunk {
                                        str: chunk.str
                                            [..num_remaining_bytes_in_in_progress_to_print]
                                            .to_smolstr(),
                                        style: Style {
                                            color: chunk.style.color,
                                            background_color: in_progress_old_row_chunk
                                                .style
                                                .background_color,
                                        },
                                    });
                                    num_bytes_already_printed_in_this_new_chunk +=
                                        num_remaining_bytes_in_in_progress_to_print;
                                    num_bytes_fully_past_in_old_row +=
                                        in_progress_old_row_chunk_len;
                                    in_progress_old_row_index + 1
                                }
                            };
                            loop {
                                let Some(next_old_chunk) = onto[row_index].get(next_old_row_index)
                                else {
                                    new_row.push(
                                        match num_bytes_already_printed_in_this_new_chunk {
                                            0 => chunk,
                                            _ => StyledChunk {
                                                str: chunk.str
                                                    [num_bytes_already_printed_in_this_new_chunk..]
                                                    .to_smolstr(),
                                                style: chunk.style,
                                            },
                                        },
                                    );
                                    break 'outer match next_old_row_index {
                                        0 => None,
                                        next_old_row_index => Some(ProgressInOldRow::FullyPast(
                                            next_old_row_index - 1,
                                        )),
                                    };
                                };
                                let next_old_chunk_len = next_old_chunk.str.len();
                                let end_byte_of_next_old_row_chunk =
                                    num_bytes_fully_past_in_old_row + next_old_chunk_len;
                                match end_byte_of_next_old_row_chunk.cmp(&end_byte_of_new_chunk) {
                                    Ordering::Equal => {
                                        new_row.push(StyledChunk {
                                            str: match num_bytes_already_printed_in_this_new_chunk {
                                                0 => chunk.str,
                                                _ => chunk.str
                                                    [num_bytes_already_printed_in_this_new_chunk..]
                                                    .to_smolstr(),
                                            },
                                            style: Style {
                                                color: chunk.style.color,
                                                background_color: next_old_chunk
                                                    .style
                                                    .background_color,
                                            },
                                        });
                                        num_bytes_fully_past_in_old_row += next_old_chunk_len;
                                        break 'outer Some(ProgressInOldRow::FullyPast(
                                            next_old_row_index,
                                        ));
                                    }
                                    Ordering::Less => {
                                        num_bytes_fully_past_in_old_row += next_old_chunk_len;
                                        new_row.push(StyledChunk {
                                            str: chunk.str
                                                [num_bytes_already_printed_in_this_new_chunk
                                                    ..num_bytes_already_printed_in_this_new_chunk
                                                        + next_old_chunk_len]
                                                .to_smolstr(),
                                            style: Style {
                                                color: chunk.style.color,
                                                background_color: next_old_chunk
                                                    .style
                                                    .background_color,
                                            },
                                        });
                                        num_bytes_already_printed_in_this_new_chunk +=
                                            next_old_chunk_len;
                                        next_old_row_index += 1;
                                    }
                                    Ordering::Greater => {
                                        new_row.push(StyledChunk {
                                            str: match num_bytes_already_printed_in_this_new_chunk {
                                                0 => chunk.str,
                                                _ => chunk.str
                                                    [num_bytes_already_printed_in_this_new_chunk..]
                                                    .to_smolstr(),
                                            },
                                            style: Style {
                                                color: chunk.style.color,
                                                background_color: next_old_chunk
                                                    .style
                                                    .background_color,
                                            },
                                        });
                                        break 'outer Some(ProgressInOldRow::InProgress(
                                            next_old_row_index,
                                        ));
                                    }
                                }
                            }
                        };
                    }
                }
            }
            num_bytes_already_seen_in_new_row += chunk_len;
        }
    }
}

#[derive(Default)]
struct ComponentHolder<'a> {
    store: UnsafeCell<Vec<Vec<Component<'a>>>>,
    len: Cell<usize>,
}

impl<'a> ComponentHolder<'a> {
    pub const PER_ROW: usize = 20;

    pub fn push(&self, component: Component<'a>) {
        let store_index = self.len.get() / Self::PER_ROW;
        // SAFETY: this should be fine because we're making sure
        // that nothing in the store moves after it's been added,
        // and the only thing that holds references to us is
        // `.get_most_recent()`
        unsafe {
            if self.len.get() % Self::PER_ROW == 0 {
                self.store_mut().push(Vec::with_capacity(Self::PER_ROW));
            }
            self.store_mut()[store_index].push(component);
        }
        self.len.set(self.len.get() + 1);
    }

    pub fn get_most_recent(&self) -> &Component<'a> {
        let len = self.len.get();
        assert!(len > 0);
        let store_index = (len - 1) / Self::PER_ROW;
        let index_in_row = (len - 1) % Self::PER_ROW;
        unsafe { &self.store_ref()[store_index][index_in_row] }
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

    unsafe fn store_ref(&self) -> &Vec<Vec<Component<'a>>> {
        unsafe { &*self.store.get() }
    }

    unsafe fn store_mut(&self) -> &mut Vec<Vec<Component<'a>>> {
        unsafe { &mut *self.store.get() }
    }
}
