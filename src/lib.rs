pub use proc_macros::soft;

mod component;
mod crossterm;
mod cursor;
mod error;
mod flex;
mod render;
mod text;

pub use component::{Component, ComponentInterface};
pub use crossterm::{size, take_over_screen, Size, TakeOverScreenGuard};
pub use cursor::{Cursor, Offset};
pub use error::Error;
pub use flex::{FlexColumn, FlexColumnBuilder};
pub use render::{Grid, Renderer};
pub use text::{Text, TextBuilder, TextChild, TextChildren};
