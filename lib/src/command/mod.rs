pub mod protocol;

mod error;
mod kind;
mod r#impl;

pub use self::{
    error::{Error as CommandError, Result as CommandResult},
    kind::{CommandType, InvalidCommandType},
};

use super::state::State;
use alloc::vec::Vec;
use protocol::CommandInfo;
use r#impl::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentNotation {
    Multiple,
    None,
    One,
}

pub trait Command<'a> {
    fn new(state: &'a State) -> Self
    where
        Self: Sized;
    fn dispatch(self, req: Request) -> CommandResult<Response>;
}

pub struct Request<'a> {
    pub args: Option<&'a [Vec<u8>]>,
}

impl Request<'_> {
    pub fn arg(&self, idx: usize) -> Option<&[u8]> {
        self.args.and_then(|args| args.get(idx)).map(|x| x.as_ref())
    }

    pub fn flatten_args(self) -> Option<Vec<u8>> {
        Some(self.args?.iter().fold(Vec::new(), |mut acc, arg| {
            acc.extend_from_slice(arg);

            acc
        }))
    }

    pub fn key(&mut self) -> Option<&[u8]> {
        self.args.and_then(|args| args.get(0).map(|x| x.as_slice()))
    }
}

#[derive(Debug)]
pub struct Response(Vec<u8>);

impl Response {
    pub fn bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    fn from_int(value: i64) -> Self {
        let mut bytes = value.to_be_bytes().to_vec();
        bytes.push(b'\n');

        Self(bytes)
    }

    fn from_usize(value: usize) -> Self {
        let mut bytes = value.to_be_bytes().to_vec();
        bytes.push(b'\n');

        Self(bytes)
    }
}

impl<T: Into<Vec<u8>>> From<T> for Response {
    fn from(v: T) -> Self {
        let mut vec: Vec<u8> = v.into();

        if !vec.ends_with(&[b'\n']) {
            vec.push(b'\n');
        }

        Self(vec)
    }
}

pub fn dispatch(state: &State, command: &CommandInfo) -> CommandResult<Response> {
    let req = Request {
        args: command.arguments.as_deref(),
    };

    match command.kind {
        CommandType::Append => Append::new(state).dispatch(req),
        CommandType::DecrementIntBy => DecrementIntBy::new(state).dispatch(req),
        CommandType::DecrementInt => DecrementInt::new(state).dispatch(req),
        CommandType::Echo => Echo::new(state).dispatch(req),
        CommandType::IncrementInt => IncrementInt::new(state).dispatch(req),
        CommandType::IncrementIntBy => IncrementIntBy::new(state).dispatch(req),
        CommandType::Stats => Stats::new(state).dispatch(req),
        CommandType::StringLength => StringLength::new(state).dispatch(req),
    }
}

#[cfg(test)]
mod tests {
    use super::Response;
    use alloc::borrow::ToOwned;

    #[test]
    fn test_response_int() {
        assert_eq!(Response::from_int(7).0, [0, 0, 0, 0, 0, 0, 0, 7].to_owned());
        assert_eq!(
            Response::from_int(68125).0,
            [0, 0, 0, 0, 0, 1, 10, 29].to_owned()
        );
    }
}
