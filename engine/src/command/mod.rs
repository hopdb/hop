pub(crate) mod r#impl;
pub mod request;
pub mod response;

mod command_id;
mod error;

pub use self::{
    command_id::{CommandId, InvalidCommandId},
    error::{Error as DispatchError, Result as DispatchResult},
    request::Request,
    response::Response,
};

use crate::Hop;
use alloc::vec::Vec;

pub trait Dispatch {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Vec<u8>>;
}

enum ContextConclusion<T> {
    Finished(T),
    Incomplete,
    Next,
}
