use core::fmt;

use crate::Code;

#[derive(Debug)]
pub enum Error {
    IO(fmt::Error),
    IncompleteResponse,
    InvalidCode([u8; 3]),
    InvalidLineOp,
    InvalidNumber(btoi::ParseIntegerError),
    UnexpectedCode(Code),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IO(err) => err.fmt(f),
            Error::IncompleteResponse => write!(f, "incomplete response"),
            Error::InvalidCode(code) => {
                write!(f, "invalid reply code {:?}", core::str::from_utf8(code))
            }
            Error::InvalidLineOp => write!(f, "expected either '-' or ' '"),
            Error::UnexpectedCode(code) => write!(f, "unexpected reply code {:?}", code),
            Error::InvalidNumber(err) => err.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Self {
        Self::IO(err)
    }
}

impl From<btoi::ParseIntegerError> for Error {
    fn from(err: btoi::ParseIntegerError) -> Self {
        Self::InvalidNumber(err)
    }
}
