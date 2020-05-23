pub mod exists;
pub mod is;
pub mod set;

mod decrement;
mod delete;
mod echo;
mod increment;
mod keys;
mod rename;
mod stats;

pub use self::{
    decrement::Decrement,
    delete::Delete,
    echo::Echo,
    exists::{Exists, ExistsConfigured},
    increment::Increment,
    is::Is,
    keys::Keys,
    rename::Rename,
    set::{SetBytes, SetUnconfigured},
    stats::Stats,
};

use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
};
use std::error::Error;

type MaybeInFlightFuture<'a, Ok, Err> = Option<Pin<Box<dyn Future<Output = Result<Ok, Err>> + 'a>>>;

#[derive(Clone, Debug)]
pub enum CommandConfigurationError {
    NoKeys,
    TooManyKeys,
}

impl Display for CommandConfigurationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NoKeys => f.write_str("no keys were provided"),
            Self::TooManyKeys => f.write_str("too many keys were provided"),
        }
    }
}

impl Error for CommandConfigurationError {}
