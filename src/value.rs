use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;

mod null;
mod boolean;
mod string;

pub type MarkerResult = Result<u8, ValueError>;

pub trait Value {
    fn get_marker(&self) -> MarkerResult;
}

#[derive(Debug)]
pub struct ValueError {
    message: String,
}

impl ValueError {
    pub fn new(message: String) -> Self {
        ValueError { message }
    }
}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ValueError {}
