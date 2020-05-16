mod decrement;
mod echo;
mod increment;

pub use self::{decrement::Decrement, echo::Echo, increment::Increment};

use std::{future::Future, pin::Pin};

type MaybeInFlightFuture<'a, Ok, Err> = Option<Pin<Box<dyn Future<Output = Result<Ok, Err>> + 'a>>>;
