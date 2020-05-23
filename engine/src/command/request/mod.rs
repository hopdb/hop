mod context;

pub use context::{Context, ParseError};

use super::CommandId;
use crate::state::{KeyType, Value};
use alloc::{
    borrow::ToOwned,
    vec::{Drain, Vec},
};
use core::{
    convert::TryInto,
    ops::{Bound, RangeBounds},
    slice::SliceIndex,
    str,
};
use dashmap::{DashMap, DashSet};

pub trait Argument<'a> {
    fn convert(bytes: &'a [u8]) -> Option<Self>
    where
        Self: Sized;
}

pub trait MultiArgument<'a> {
    fn convert(arguments: &'a [Vec<u8>]) -> Option<Self>
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
    fn convert(arguments: &[Vec<u8>]) -> Option<Self> {
        let map = DashMap::new();
        let mut args = arguments.iter();

        while let (Some(k), Some(v)) = (args.next(), args.next()) {
            map.insert(k.to_owned(), v.to_owned());
        }

        Some(map)
    }
}

impl MultiArgument<'_> for DashSet<Vec<u8>> {
    fn convert(arguments: &[Vec<u8>]) -> Option<Self> {
        let set = DashSet::new();

        for argument in arguments {
            set.insert(argument.to_owned());
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

    pub fn args<I: SliceIndex<[Vec<u8>]>>(&self, index: I) -> Option<&I::Output> {
        self.args.as_ref()?.get(index)
    }

    pub fn typed_args<'a, T: MultiArgument<'a>>(&'a self) -> Option<T> {
        let args = self.args.as_ref()?;

        T::convert(args.get(1..)?)
    }

    pub fn arg(&self, idx: usize) -> Option<&[u8]> {
        self.args.as_ref()?.get(idx).map(AsRef::as_ref)
    }

    pub fn arg_count(&self) -> usize {
        self.args.as_ref().map(|args| args.len()).unwrap_or(0)
    }

    pub fn typed_arg<'a, T: Argument<'a>>(&'a self, index: usize) -> Option<T> {
        let args = self.args.as_ref()?;
        let arg = args.get(index)?;

        T::convert(arg)
    }

    pub fn take_args(&mut self, range: impl RangeBounds<usize>) -> Option<Drain<'_, Vec<u8>>> {
        let start = match range.start_bound() {
            Bound::Excluded(amt) => *amt + 1,
            Bound::Included(amt) => *amt,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Excluded(amt) => *amt,
            Bound::Included(amt) => *amt + 1,
            Bound::Unbounded => match self.args.as_ref() {
                Some(args) => args.len(),
                None => return None,
            },
        };

        Some(self.args.as_mut()?.drain(start..end))
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

pub fn write_value_to_args(value: Value, to: &mut Vec<Vec<u8>>) {
    match value {
        Value::Boolean(bool) => {
            let mut buf = Vec::with_capacity(1);
            buf.push(bool as u8);

            to.push(buf);
        }
        Value::Bytes(bytes) => to.push(bytes),
        Value::Float(float) => to.push(float.to_be_bytes().to_vec()),
        Value::Integer(int) => to.push(int.to_be_bytes().to_vec()),
        Value::List(list) => {
            for item in list {
                to.push(item);
            }
        }
        Value::Map(map) => {
            for (k, v) in map.into_iter() {
                to.push(k);
                to.push(v);
            }
        }
        Value::Set(set) => {
            for item in set {
                to.push(item);
            }
        }
        Value::String(string) => to.push(string.into_bytes()),
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
