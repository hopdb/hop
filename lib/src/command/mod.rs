pub(crate) mod r#impl;
pub mod request;
pub mod response;

mod command_id;
mod error;

pub use self::{
    error::{Error as CommandError, Result as CommandResult},
    command_id::{CommandType, InvalidCommandType},
    request::Request,
    response::Response,
};

use crate::Hop;

pub trait Dispatch {
    fn dispatch(hop: &Hop, req: &mut Request) -> CommandResult<Response>;
}

#[cfg(test)]
mod tests {
    use super::Response;
    use alloc::borrow::ToOwned;

    #[test]
    fn test_response_int() {
        assert_eq!(
            Response::from_int(7).into_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 7, b'\n'].to_owned()
        );
        assert_eq!(
            Response::from_int(68125).into_bytes(),
            [0, 0, 0, 0, 0, 1, 10, 29, b'\n'].to_owned()
        );
    }
}
