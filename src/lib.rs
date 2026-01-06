pub use proc_macros::soft;

pub use anyhow;
pub use crossterm;

mod backend;
mod component;
mod cursor;
mod error;
mod flex;
mod layout;
mod render;
mod style;
mod terminal;
mod text;

pub use backend::{
    Backend, BackendCrossterm, BackendInterface, BackendMemory, Position, RowOrColumnNumber, Size,
};
pub use component::{Component, ComponentInterface};
pub use cursor::{Cursor, Offset};
pub use error::Error;
pub use flex::{FlexColumn, FlexColumnBuilder, FlexRow, FlexRowBuilder};
pub use layout::{Absolute, Overflow, Relative};
pub use render::{Grid, Renderer, RendererBuilder};
pub use style::{Style, StyleBuilder};
pub use terminal::{take_over_screen, TakeOverScreenGuard};
pub use text::{Text, TextBuilder, TextChild, TextChildren};
