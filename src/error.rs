use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TextBuilder: {0}")]
    TextBuilder(String),
    #[error("FlexColumnBuilder: {0}")]
    FlexColumnBuilder(String),
    #[error("crossterm: {0}")]
    Crossterm(String),
    #[error("Rendered cursor more than once")]
    RenderedCursorMoreThanOnce,
}
