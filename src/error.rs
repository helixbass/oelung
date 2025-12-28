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
    #[error("{0}")]
    Anyhow(anyhow::Error),
    #[error("Cannot render a non-text child inside a Text component")]
    RenderedNonTextChildInText,
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self::Anyhow(value)
    }
}
