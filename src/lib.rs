pub use proc_macros::soft;

pub use anyhow;
pub use crossterm;

mod component;
mod cursor;
mod error;
mod flex;
mod layout;
mod render;
mod style;
mod terminal;
mod text;

pub use component::{Component, ComponentInterface};
pub use cursor::{Cursor, Offset};
pub use error::Error;
pub use flex::{FlexColumn, FlexColumnBuilder};
pub use layout::{Absolute, Overflow, Relative};
pub use render::{Grid, Renderer};
pub use style::{Style, StyleBuilder};
pub use terminal::{size, take_over_screen, Size, TakeOverScreenGuard};
pub use text::{Text, TextBuilder, TextChild, TextChildren};
