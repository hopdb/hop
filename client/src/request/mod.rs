pub mod exists;
pub mod set;

mod decrement;
mod delete;
mod echo;
mod increment;
mod rename;
mod stats;

pub use self::{
    decrement::Decrement,
    delete::Delete,
    echo::Echo,
    exists::{Exists, ExistsConfigured},
    increment::Increment,
    rename::Rename,
    set::{SetBytes, SetUnconfigured},
    stats::Stats,
};

use std::{future::Future, pin::Pin};

type MaybeInFlightFuture<'a, Ok, Err> = Option<Pin<Box<dyn Future<Output = Result<Ok, Err>> + 'a>>>;
