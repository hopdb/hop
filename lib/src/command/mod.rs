pub mod protocol;

mod error;
mod r#impl;

pub use error::{Error as CommandError, Result as CommandResult};

use super::state::State;
use alloc::vec::Vec;
use core::convert::TryFrom;
use protocol::CommandInfo;
use r#impl::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentNotation {
    Multiple,
    None,
    One,
}

#[derive(Debug)]
pub struct InvalidCommandType;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CommandType {
    IncrementInt = 0,
    DecrementInt = 1,
    IncrementIntBy = 2,
    DecrementIntBy = 3,
    Append = 20,
    StringLength = 21,
    Echo = 100,
    Stats = 101,
}

impl CommandType {
    pub fn argument_notation(self) -> ArgumentNotation {
        use ArgumentNotation::*;
        use CommandType::*;

        match self {
            Append => One,
            Echo => Multiple,
            IncrementInt => None,
            IncrementIntBy => One,
            DecrementInt => None,
            DecrementIntBy => One,
            Stats => None,
            StringLength => One,
        }
    }

    pub fn has_key(self) -> bool {
        use CommandType::*;

        match self {
            Stats => false,
            _ => true,
        }
    }

    pub fn is_simple(self) -> bool {
        self.argument_notation() == ArgumentNotation::None && !self.has_key()
    }
}

impl TryFrom<u8> for CommandType {
    type Error = InvalidCommandType;

    fn try_from(num: u8) -> Result<Self, Self::Error> {
        Ok(match num {
            0 => CommandType::IncrementInt,
            1 => CommandType::DecrementInt,
            _ => return Err(InvalidCommandType),
        })
    }
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

    pub fn flatten_args(self) -> Vec<u8> {
        // self.args.unwrap().cloned().flatten().collect()
        Vec::new()
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

    fn from_int(v: i64) -> Self {
        Self(v.to_be_bytes().to_vec())
    }

    fn from_usize(v: usize) -> Self {
        Self(v.to_be_bytes().to_vec())
    }
}

impl<T: Into<Vec<u8>>> From<T> for Response {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

pub fn dispatch(state: &State, command: &CommandInfo) -> CommandResult<Response> {
    let req = Request {
        args: command.arguments.as_deref(),
    };

    let mut resp = match command.kind {
        CommandType::Append => Append::new(state).dispatch(req),
        CommandType::DecrementIntBy => DecrementIntBy::new(state).dispatch(req),
        CommandType::DecrementInt => DecrementInt::new(state).dispatch(req),
        CommandType::Echo => Echo::new(state).dispatch(req),
        CommandType::IncrementInt => IncrementInt::new(state).dispatch(req),
        CommandType::IncrementIntBy => IncrementIntBy::new(state).dispatch(req),
        CommandType::Stats => Stats::new(state).dispatch(req),
        CommandType::StringLength => StringLength::new(state).dispatch(req),
    }?;

    resp.0.push(b'\n');

    Ok(resp)
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
