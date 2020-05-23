pub mod command_id;
pub(crate) mod r#impl;
pub mod request;
pub mod response;

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
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()>;
}

enum ContextConclusion<T> {
    Finished(T),
    Incomplete,
    Next,
}
