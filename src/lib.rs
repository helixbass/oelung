pub use proc_macros::soft;

mod component;
mod crossterm;
mod cursor;
mod error;
mod render;
mod text;

pub use component::Component;
pub use crossterm::{size, take_over_screen, Size, TakeOverScreenGuard};
pub use cursor::{Cursor, Offset};
pub use error::Error;
pub use render::Renderer;
pub use text::{Text, TextBuilder, TextChild, TextChildren};
