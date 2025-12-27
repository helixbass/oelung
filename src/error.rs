use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TextBuilder: {0}")]
    TextBuilder(String),
    #[error("crossterm: {0}")]
    Crossterm(String),
}
