#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

extern crate alloc;

mod parse;
mod print;
mod process;

pub use self::{
    parse::{parse, ParseError},
    process::{process, ProcessError},
};
