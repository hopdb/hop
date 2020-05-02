pub(crate) mod r#impl;
pub mod request;
pub mod response;

mod command_id;
mod error;

pub use self::{
    command_id::{CommandId, InvalidCommandId},
    error::{Error as CommandError, Result as CommandResult},
    request::Request,
    response::Response,
};

use crate::Hop;

pub trait Dispatch {
    fn dispatch(hop: &Hop, req: &Request) -> CommandResult<Response>;
}
