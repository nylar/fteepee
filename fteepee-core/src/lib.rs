#![cfg_attr(not(feature = "std"), no_std)]

mod code;
pub mod commands;
mod error;
mod parsers;
pub mod response;

#[cfg(feature = "std")]
extern crate std as core;

pub use crate::code::Code;
pub use crate::error::Error;

#[cfg(feature = "std")]
pub use crate::parsers::parse_features;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Default)]
pub struct Config {
    pub mlst_supported: bool,
}

// TODO: Handle connection closed?
#[macro_export]
macro_rules! expect_code {
    ($reply:expr, $pat:pat $(,)?) => {
        match $reply {
            $pat => (),
            _ => Err($crate::Error::UnexpectedCode($reply))?,
        }
    };
}

pub struct Disconnected;
pub struct Connected;
