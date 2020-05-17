mod context;

pub use context::{Context, ParseError};

use super::CommandId;
use crate::state::KeyType;
use alloc::vec::Vec;
use core::slice::SliceIndex;

#[derive(Debug)]
pub struct Request {
    args: Option<Vec<Vec<u8>>>,
    key_type: Option<KeyType>,
    kind: CommandId,
}

impl Request {
    pub fn new(kind: CommandId, args: Option<Vec<Vec<u8>>>) -> Self {
        Self {
            args,
            key_type: None,
            kind,
        }
    }

    pub fn new_with_type(kind: CommandId, args: Option<Vec<Vec<u8>>>, key_type: KeyType) -> Self {
        Self {
            args,
            key_type: Some(key_type),
            kind,
        }
    }

    pub fn args(&self) -> Option<&[Vec<u8>]> {
        self.args.as_deref()
    }

    pub fn arg<I: SliceIndex<[Vec<u8>]>>(
        &self,
        index: I,
    ) -> Option<&<I as SliceIndex<[Vec<u8>]>>::Output> {
        let args = self.args.as_ref()?;

        let refs = args.get(index)?;

        Some(refs)
    }

    pub fn flatten_args(&self) -> Option<Vec<u8>> {
        let start = if self.kind.has_key() { 1 } else { 0 };

        Some(
            self.args
                .as_ref()?
                .get(start..)?
                .iter()
                .fold(Vec::new(), |mut acc, arg| {
                    acc.extend_from_slice(arg);

                    acc
                }),
        )
    }

    pub fn key(&self) -> Option<&[u8]> {
        if !self.kind.has_key() {
            return None;
        }

        self.args
            .as_ref()
            .and_then(|args| args.get(0).map(|x| x.as_slice()))
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

    pub fn kind(&self) -> CommandId {
        self.kind
    }

    pub fn into_args(mut self) -> Option<Vec<Vec<u8>>> {
        self.args.take()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut vec = Vec::new();
        let mut byte = self.kind as u8;

        if self.key_type.is_some() {
            byte |= 0b1000_0000;
        }

        vec.push(byte);

        if let Some(key_type) = self.key_type {
            vec.push(key_type as u8);
        }

        let args = match self.args {
            Some(args) => args,
            None => return vec,
        };

        vec.push(args.len() as u8);

        for arg in args {
            let arg_len = arg.len() as u32;

            vec.extend_from_slice(&arg_len.to_be_bytes());
            vec.extend_from_slice(&arg);
        }

        vec
    }
}

#[cfg(test)]
mod tests {
    use super::{super::CommandId, Request};
    use crate::state::KeyType;
    use alloc::vec::Vec;
    use core::fmt::Debug;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Request: Debug);

    #[test]
    fn test_request_into_bytes_simple() {
        let req = Request::new(CommandId::Stats, None);
        assert_eq!(
            req.into_bytes(),
            &[
                // note bit 0 is not flipped
                0b0110_0101,
            ]
        );

        let req = Request::new_with_type(CommandId::Increment, None, KeyType::Float);
        assert_eq!(
            req.into_bytes(),
            &[
                // now that we specify a key type, bit 0 is flipped
                0b1000_0000,
                KeyType::Float as u8,
            ]
        );
    }

    #[test]
    fn test_request_into_bytes_echo() {
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push(b"bar".to_vec());
        let req = Request::new(CommandId::Echo, Some(args));
        assert_eq!(
            req.into_bytes(),
            &[
                // no key type
                CommandId::Echo as u8,
                // number of arguments
                2,
                // argument 1 length
                0,
                0,
                0,
                3,
                // argument 1
                b'f',
                b'o',
                b'o',
                // argument 2 length
                0,
                0,
                0,
                3,
                // argument 2
                b'b',
                b'a',
                b'r',
            ]
        );
    }

    #[test]
    fn test_request_into_bytes_increment() {
        let mut args = Vec::new();
        args.push(b"key".to_vec());
        let req = Request::new_with_type(CommandId::Increment, Some(args), KeyType::Integer);
        assert_eq!(
            req.into_bytes(),
            &[
                // key type is specified
                0b1000_0000 | CommandId::Increment as u8,
                // key type
                KeyType::Integer as u8,
                // number of arguments
                1,
                // argument 1 length
                0,
                0,
                0,
                3,
                // argument 1
                b'k',
                b'e',
                b'y',
            ]
        );
    }
}
