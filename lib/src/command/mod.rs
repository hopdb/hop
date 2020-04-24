pub mod protocol;

mod error;
mod r#impl;

pub use error::{Error as CommandError, Result as CommandResult};

use r#impl::*;
use protocol::CommandInfo;
use std::convert::TryFrom;
use super::state::State;

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
    Stats = 100,
}

impl CommandType {
    pub fn argument_notation(&self) -> ArgumentNotation {
        use ArgumentNotation::*;
        use CommandType::*;

        match self {
            IncrementInt => None,
            IncrementIntBy => One,
            DecrementInt => None,
            DecrementIntBy => None,
            Stats => None,
        }
    }

    pub fn has_key(&self) -> bool {
        use CommandType::*;

        match self {
            Stats => false,
            _ => true,
        }
    }

    pub fn is_simple(&self) -> bool {
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
    fn new(state: &'a mut State) -> Self where Self: Sized;
    fn dispatch(self, req: Request) -> CommandResult<Response>;
}

pub struct Request<'a> {
    pub args: Option<&'a [Vec<u8>]>,
}

impl Request<'_> {
    pub fn flatten_args(self) -> Vec<u8> {
        // self.args.unwrap().cloned().flatten().collect()
        vec![]
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
}

impl<T: Into<Vec<u8>>> From<T> for Response {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

pub fn dispatch(state: &mut State, command: &CommandInfo) -> CommandResult<Response> {
    let req = Request {
        args: command.arguments.as_ref().map(|x| x.as_slice()),
    };

    let mut resp = match command.kind {
        CommandType::DecrementInt => {
            DecrementInt::new(state).dispatch(req)
        }
        CommandType::IncrementInt => {
            IncrementInt::new(state).dispatch(req)
        },
        CommandType::IncrementIntBy => {
            IncrementIntBy::new(state).dispatch(req)
        },
        _ => unimplemented!(),
    }?;

    resp.0.push(b'\n');

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn test_response_int() {
        assert_eq!(Response::from_int(7).0, vec![0, 0, 0, 0, 0, 0, 0, 7]);
        assert_eq!(Response::from_int(68125).0, vec![0, 0, 0, 0, 0, 1, 10, 29]);
    }
}
