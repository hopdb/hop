pub mod append;
pub mod exists;
pub mod get;
pub mod is;
pub mod set;

mod decrement;
mod delete;
mod echo;
mod increment;
mod keys;
mod length;
mod rename;
mod stats;
mod r#type;

pub use self::{
    decrement::Decrement,
    delete::Delete,
    echo::Echo,
    exists::{Exists, ExistsConfigured},
    increment::Increment,
    is::Is,
    keys::Keys,
    length::Length,
    r#type::Type,
    rename::Rename,
    stats::Stats,
};

use alloc::boxed::Box;
use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
};

type MaybeInFlightFuture<'a, Ok, Err> =
    Option<Pin<Box<dyn Future<Output = Result<Ok, Err>> + Send + 'a>>>;

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

#[cfg(feature = "std")]
impl std::error::Error for CommandConfigurationError {}

#[cfg(test)]
mod tests {
    use super::CommandConfigurationError;
    use core::fmt::Debug;
    use static_assertions::assert_impl_all;

    assert_impl_all!(CommandConfigurationError: Clone, Debug, Send);
}
