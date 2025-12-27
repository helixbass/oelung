use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TextBuilder: {0}")]
    TextBuilder(String),
}
