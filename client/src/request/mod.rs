mod decrement;
mod echo;
mod increment;
mod stats;

pub use self::{decrement::Decrement, echo::Echo, increment::Increment, stats::Stats};

use std::{future::Future, pin::Pin};

type MaybeInFlightFuture<'a, Ok, Err> = Option<Pin<Box<dyn Future<Output = Result<Ok, Err>> + 'a>>>;
