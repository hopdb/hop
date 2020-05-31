mod builder;
mod context;

pub use self::{
    builder::{RequestBuilder, RequestBuilderError},
    context::{Context, ParseError},
};

use super::command_id::{CommandId, KeyNotation};
use crate::state::KeyType;
use alloc::{
    borrow::{Cow, ToOwned},
    vec::Vec,
};
use arrayvec::ArrayVec;
use core::{
    convert::TryInto,
    ops::{Bound, RangeBounds},
    str,
};
use dashmap::{DashMap, DashSet};

pub trait Argument<'a> {
    fn convert(bytes: &'a [u8]) -> Option<Self>
    where
        Self: Sized;
}

pub trait MultiArgument<'a> {
    fn convert(args: Arguments<'a>) -> Option<Self>
    where
        Self: Sized;
}

impl<'a> Argument<'a> for &'a [u8] {
    fn convert(bytes: &'a [u8]) -> Option<Self> {
        Some(bytes)
    }
}

impl Argument<'_> for bool {
    fn convert(bytes: &[u8]) -> Option<Self> {
        let byte = bytes.first()?;

        Some(*byte > 0)
    }
}

impl Argument<'_> for f64 {
    fn convert(bytes: &[u8]) -> Option<Self> {
        let arr = bytes.get(..8)?.try_into().ok()?;

        Some(f64::from_be_bytes(arr))
    }
}

impl Argument<'_> for i64 {
    fn convert(bytes: &[u8]) -> Option<Self> {
        let arr = bytes.get(..8)?.try_into().ok()?;

        Some(i64::from_be_bytes(arr))
    }
}

impl MultiArgument<'_> for DashMap<Vec<u8>, Vec<u8>> {
    fn convert(mut args: Arguments<'_>) -> Option<Self> {
        let map = DashMap::new();

        while let (Some(k), Some(v)) = (args.next(), args.next()) {
            map.insert(k.to_owned(), v.to_owned());
        }

        Some(map)
    }
}

impl MultiArgument<'_> for DashSet<Vec<u8>> {
    fn convert(args: Arguments<'_>) -> Option<Self> {
        let set = DashSet::new();

        for arg in args {
            set.insert(arg.to_owned());
        }

        Some(set)
    }
}

impl<'a> Argument<'a> for &'a str {
    fn convert(bytes: &'a [u8]) -> Option<Self> {
        str::from_utf8(bytes).ok()
    }
}

#[derive(Debug)]
pub struct Arguments<'a> {
    idx: usize,
    to: usize,
    request: &'a Request<'a>,
}

impl ExactSizeIterator for Arguments<'_> {
    fn len(&self) -> usize {
        self.to
    }
}

impl<'a> Iterator for Arguments<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.to {
            return None;
        }

        let idx = self.idx;
        self.idx += 1;

        self.request.arg(idx)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request<'a> {
    buf: Cow<'a, [u8]>,
    command_id: CommandId,
    key_type: Option<KeyType>,
    positions: Cow<'a, ArrayVec<[usize; 256]>>,
}

impl<'a> Request<'a> {
    pub fn command_id(&self) -> CommandId {
        self.command_id
    }

    pub fn args(&self, range: impl RangeBounds<usize>) -> Option<Arguments<'_>> {
        if self.arg_count() == 0 {
            return None;
        }

        let start = match range.start_bound() {
            Bound::Excluded(amt) => *amt + 1,
            Bound::Included(amt) => *amt,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Excluded(amt) => *amt,
            Bound::Included(amt) => *amt + 1,
            Bound::Unbounded => self.arg_count(),
        };

        Some(Arguments {
            idx: start,
            request: self,
            to: end,
        })
    }

    pub fn typed_args<'b, T: MultiArgument<'b>>(&'b self) -> Option<T> {
        let args = self.args(1..)?;

        T::convert(args)
    }

    pub fn arg(&self, idx: usize) -> Option<&[u8]> {
        let position = self.positions.get(idx).copied()?;

        if idx == 0 {
            let base = if self.key_type.is_some() { 1 } else { 0 };

            return self.buf.get(6 + base..=position);
        }

        let previous = self.positions.get(idx - 1)?;

        self.buf.get(previous + 5..=position)
    }

    pub fn arg_count(&self) -> usize {
        self.positions.len()
    }

    pub fn typed_arg<'b, T: Argument<'b>>(&'b self, idx: usize) -> Option<T> {
        let arg = self.arg(idx)?;

        T::convert(arg)
    }

    pub fn key(&self) -> Option<&[u8]> {
        if self.command_id.key_notation() == KeyNotation::None {
            return None;
        }

        self.arg(0)
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

    // pub fn into_args(mut self) -> Option<Vec<Vec<u8>>> {
    //     self.args.take()
    // }

    pub fn into_bytes(self) -> Cow<'a, [u8]> {
        self.buf
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.buf.as_ref()
    }
}

impl From<RequestBuilder> for Request<'_> {
    fn from(builder: RequestBuilder) -> Self {
        builder.into_request()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::CommandId, Request, RequestBuilder};
    use crate::state::KeyType;
    use core::fmt::Debug;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Request: Debug);

    #[test]
    fn test_request_into_bytes_simple() {
        let req = RequestBuilder::new(CommandId::Stats).into_request();
        assert_eq!(
            req.into_bytes().as_ref(),
            &[
                // note bit 0 is not flipped
                0b0110_0101,
            ]
        );

        let builder = RequestBuilder::new_with_key_type(CommandId::Increment, KeyType::Float);
        let req = builder.into_request();

        assert_eq!(
            req.buf.as_ref(),
            &[
                // now that we specify a key type, bit 0 is flipped
                0b1000_0000,
                KeyType::Float as u8,
                0,
            ]
        );
    }

    #[test]
    fn test_args() {
        let mut builder = RequestBuilder::new(CommandId::Decrement);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());

        let req = builder.into_request();
        let mut args = req.args(..).unwrap();

        assert_eq!(Some(b"foo".as_ref()), args.next());
    }

    #[test]
    fn test_args_many() {
        let mut builder = RequestBuilder::new(CommandId::Echo);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(b"bar".as_ref()).is_ok());
        assert!(builder.bytes(b"baz".as_ref()).is_ok());
        let req = builder.into_request();
        let mut args = req.args(..).unwrap();

        assert_eq!(Some(b"foo".as_ref()), args.next());
        assert_eq!(Some(b"bar".as_ref()), args.next());
        assert_eq!(Some(b"baz".as_ref()), args.next());
    }

    #[test]
    fn test_request_into_bytes_echo() {
        let mut builder = RequestBuilder::new(CommandId::Echo);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(b"bar".as_ref()).is_ok());
        let req = builder.into_request();

        assert_eq!(
            req.buf.as_ref(),
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
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Increment, KeyType::Integer);
        assert!(builder.bytes(b"key".as_ref()).is_ok());
        let req = builder.into_request();
        assert_eq!(
            req.into_bytes().as_ref(),
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
