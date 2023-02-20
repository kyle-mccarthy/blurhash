#![deny(clippy::all)]

mod base83;
pub mod encode;
pub mod error;

pub use encode::encode;
pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Component {
    x: u8,
    y: u8,
}

impl Component {
    pub fn try_new(x: u8, y: u8) -> Result<Self> {
        if !(1..=9).contains(&x) {
            return Err(Error::ComponentOutOfBounds(x));
        }

        if !(1..=9).contains(&y) {
            return Err(Error::ComponentOutOfBounds(y));
        }

        Ok(Self { x, y })
    }

    #[inline]
    pub fn x(&self) -> u8 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> u8 {
        self.y
    }
}
