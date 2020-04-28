pub(crate) mod r#impl;
pub mod protocol;

mod error;
mod kind;

pub use self::{
    error::{Error as CommandError, Result as CommandResult},
    kind::{CommandType, InvalidCommandType},
};

use alloc::vec::Vec;
use crate::{state::KeyType, Hop};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentNotation {
    Multiple,
    None,
    One,
}

pub trait Dispatch {
    fn dispatch(hop: &Hop, req: &mut Request) -> CommandResult<Response>;
}

pub struct Request {
    args: Option<Vec<Vec<u8>>>,
    key_type: Option<KeyType>,
    kind: CommandType,
}

impl Request {
    pub fn new(kind: CommandType, args: Option<Vec<Vec<u8>>>) -> Self {
        Self {
            args,
            key_type: None,
            kind,
        }
    }

    pub fn arg(&self, idx: usize) -> Option<&[u8]> {
        self.args.as_ref().and_then(|args| args.get(idx)).map(|x| x.as_ref())
    }

    pub fn flatten_args(&self) -> Option<Vec<u8>> {
        let start = if self.kind.has_key() { 1 } else { 0 };

        Some(self.args.as_ref()?.get(start..)?.iter().fold(Vec::new(), |mut acc, arg| {
            acc.extend_from_slice(arg);

            acc
        }))
    }

    pub fn key(&mut self) -> Option<&[u8]> {
        if !self.kind.has_key() {
            return None;
        }

        self.args.as_ref().and_then(|args| args.get(0).map(|x| x.as_slice()))
    }

    /// Returns the requested type of key to work with, if any.
    ///
    /// Some commands only work with one type of key, such as a boolean, where
    /// this isn't taken into account. Other commands, such as [`Append`], can
    /// work with bytes, lists, and strings in unique ways. Commands like
    /// `Append` check the key type to know what type of key to work with.
    ///
    /// [`Append`]: impl/struct.Append.html
    pub fn key_type(&self) -> Option<KeyType> {
        self.key_type
    }

    pub fn kind(&self) -> CommandType {
        self.kind
    }

    pub fn into_args(mut self) -> Option<Vec<Vec<u8>>> {
        self.args.take()
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

#[cfg(test)]
mod tests {
    use super::Response;
    use alloc::borrow::ToOwned;

    #[test]
    fn test_response_int() {
        assert_eq!(Response::from_int(7).0, [0, 0, 0, 0, 0, 0, 0, 7, b'\n'].to_owned());
        assert_eq!(
            Response::from_int(68125).0,
            [0, 0, 0, 0, 0, 1, 10, 29, b'\n'].to_owned()
        );
    }
}
