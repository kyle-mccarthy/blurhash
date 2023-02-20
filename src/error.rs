#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The component value must fall in the range (1..=9)")]
    ComponentOutOfBounds(u8),
}

pub type Result<T> = std::result::Result<T, Error>;
