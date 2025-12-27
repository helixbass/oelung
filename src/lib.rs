mod component;
mod crossterm;
mod error;
mod text;

pub use component::Component;
pub use crossterm::take_over_screen;
pub use error::Error;
pub use text::{Text, TextBuilder, TextChild, TextChildren};

pub fn render(component: Component) {
    unimplemented!()
}
