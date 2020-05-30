use super::Request;
use crate::{
    command::CommandId,
    state::{KeyType, Value},
};
use alloc::{borrow::Cow, vec::Vec};
use arrayvec::ArrayVec;
use core::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestBuilderError {
    ArgumentEmpty,
    TooManyArguments,
    ValueEmpty,
}

impl Display for RequestBuilderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::ArgumentEmpty => f.write_str("the provided argument is empty"),
            Self::TooManyArguments => {
                f.write_str("too many arguments have been given to the builder")
            }
            Self::ValueEmpty => f.write_str("the provided value variant is empty (0 length)"),
        }
    }
}

/// Builder to construct valid requests.
///
/// The builder is useful because it will ensure that empty arguments are not
/// provided and that not too many arguments are given.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestBuilder {
    argument_count: u8,
    buf: Vec<u8>,
    cmd_id: CommandId,
    key_type: Option<KeyType>,
    positions: ArrayVec<[usize; 256]>,
}

impl RequestBuilder {
    /// Create a new request builder.
    pub fn new(cmd_id: CommandId) -> Self {
        let mut buf = Vec::new();
        buf.push(cmd_id as u8);

        // if it's a non-simple command, push the argument len now
        if !cmd_id.is_simple() {
            buf.push(0);
        }

        Self {
            argument_count: 0,
            buf,
            cmd_id,
            key_type: None,
            positions: ArrayVec::new(),
        }
    }

    pub fn new_with_key_type(cmd_id: CommandId, key_type: impl Into<Option<KeyType>>) -> Self {
        let key_type = key_type.into();

        let mut buf = Vec::new();
        let mut byte = cmd_id as u8;

        if key_type.is_some() {
            byte |= 0b1000_0000;
        }

        buf.push(byte);

        if let Some(key_type) = key_type {
            buf.push(key_type as u8);
        }

        if !cmd_id.is_simple() {
            buf.push(0);
        }

        Self {
            argument_count: 0,
            buf,
            cmd_id,
            key_type,
            positions: ArrayVec::new(),
        }
    }

    /// Consume the builder and return the built request.
    pub fn into_request(self) -> Request<'static> {
        Request {
            buf: Cow::Owned(self.buf),
            key_type: self.key_type,
            kind: self.cmd_id,
            positions: Cow::Owned(self.positions),
        }
    }

    /// Retrieve an immutable reference to the command ID.
    pub fn command_id_ref(&self) -> &CommandId {
        &self.cmd_id
    }

    /// Retrieve an immutable reference to the key type, if any.
    pub fn key_type_ref(&self) -> Option<&KeyType> {
        self.key_type.as_ref()
    }

    /// Add an argument containing the given bytes.
    ///
    /// # Errors
    ///
    /// Returns [`RequestBuilderError::ArgumentEmpty`] if the given value is
    /// empty.
    ///
    /// Returns [`RequestBuilderError::TooManyArguments`] if the argument would
    /// not fit in the arguments list.
    ///
    /// [`RequestBuilderError::ArgumentEmpty`]: enum.RequestBuilderError.html#variant.ArgumentEmpty
    /// [`RequestBuilderError::TooManyArguments`]: enum.RequestBuilderError.html#variant.TooManyArguments
    pub fn bytes(&mut self, bytes: impl Into<Vec<u8>>) -> Result<&mut Self, RequestBuilderError> {
        self._bytes(bytes.into())
    }

    fn _bytes(&mut self, bytes: Vec<u8>) -> Result<&mut Self, RequestBuilderError> {
        if bytes.is_empty() {
            return Err(RequestBuilderError::ArgumentEmpty);
        }

        self.push_argument(bytes)?;

        Ok(self)
    }

    /// Add a value's serialised representation to the arguments.
    ///
    /// # Errors
    ///
    /// Returns [`RequestBuilderError::ArgumentEmpty`] if the given value is
    /// empty.
    ///
    /// Returns [`RequestBuilderError::TooManyArguments`] if the argument would
    /// not fit in the arguments list.
    ///
    /// Returns [`RequestBuilderError::ValueEmpty`] if the given value's
    /// bytes, list, map, set, or string variant is empty.
    ///
    /// [`RequestBuilderError::ArgumentEmpty`]: enum.RequestBuilderError.html#variant.ArgumentEmpty
    /// [`RequestBuilderError::TooManyArguments`]: enum.RequestBuilderError.html#variant.TooManyArguments
    /// [`RequestBuilderError::ValueEmpty`]: enum.RequestBuilderError.html#variant.ValueEmpty
    pub fn value(&mut self, value: impl Into<Value>) -> Result<&mut Self, RequestBuilderError> {
        self._value(value.into())
    }

    fn _value(&mut self, value: Value) -> Result<&mut Self, RequestBuilderError> {
        match value {
            Value::Boolean(bool) => {
                self.push_argument([bool as u8].as_ref())?;
            }
            Value::Bytes(bytes) => {
                if bytes.is_empty() {
                    return Err(RequestBuilderError::ValueEmpty);
                }

                self.push_argument(bytes)?;
            }
            Value::Float(float) => {
                self.push_argument(Cow::Borrowed(float.to_be_bytes().as_ref()))?
            }
            Value::Integer(int) => self.push_argument(Cow::Borrowed(int.to_be_bytes().as_ref()))?,
            Value::List(list) => {
                if self.arguments_would_overfill(list.len() as u8) {
                    return Err(RequestBuilderError::TooManyArguments);
                }

                for item in list {
                    if item.is_empty() {
                        return Err(RequestBuilderError::ValueEmpty);
                    }

                    self.push_argument(item)?;
                }
            }
            Value::Map(map) => {
                if self.arguments_would_overfill(map.len() as u8) {
                    return Err(RequestBuilderError::TooManyArguments);
                }

                if map.is_empty() {
                    return Err(RequestBuilderError::ValueEmpty);
                }

                for (k, v) in map.into_iter() {
                    self.push_argument(k)?;
                    self.push_argument(v)?;
                }
            }
            Value::Set(set) => {
                if self.arguments_would_overfill(set.len() as u8) {
                    return Err(RequestBuilderError::TooManyArguments);
                }

                if set.is_empty() {
                    return Err(RequestBuilderError::ValueEmpty);
                }

                for item in set {
                    self.push_argument(item)?;
                }
            }
            Value::String(string) => {
                if string.is_empty() {
                    return Err(RequestBuilderError::ValueEmpty);
                }

                self.push_argument(string.as_bytes())?;
            }
        }

        Ok(self)
    }

    fn arguments_would_overfill(&self, amount: u8) -> bool {
        self.argument_count.checked_add(amount).is_none()
    }

    fn update_count(&mut self) {
        self.buf[1 + self.key_type.is_some() as usize] = self.argument_count;
    }

    /// Pushes an argument to the list.
    ///
    /// # Errors
    ///
    /// Returns [`RequestBuilderError::TooManyArguments`] if the list of
    /// arguments is already full.
    ///
    /// [`RequestBuilderError::TooManyArguments`]: enum.RequestBuilderError.html#variant.TooManyArguments
    fn push_argument<'a>(
        &mut self,
        argument: impl Into<Cow<'a, [u8]>>,
    ) -> Result<(), RequestBuilderError> {
        let argument = argument.into();
        let argument_len = argument.len();

        let argument_len_bytes = (argument_len as u32).to_be_bytes();
        self.buf.extend_from_slice(&argument_len_bytes);

        match argument {
            Cow::Borrowed(arg) => self.buf.extend_from_slice(arg),
            Cow::Owned(mut arg) => {
                self.buf.append(&mut arg);
            }
        }

        self.argument_count += 1;
        self.update_count();

        let base = self.key_type.is_some() as usize;
        let position = match self.positions.last().copied() {
            Some(position) => position + 4 + argument_len,
            None => {
                // skip command id
                //
                // key type (1 if exists) + arg count + arg len as u32 + arg len
                base + 1 + 4 + argument_len
            }
        };
        self.positions.push(position);

        Ok(())
    }
}

impl From<Request<'_>> for RequestBuilder {
    fn from(request: Request) -> Self {
        let mut builder = Self::new_with_key_type(request.kind, request.key_type);
        builder.buf = request.buf.into_owned();
        builder.positions = request.positions.into_owned();

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::RequestBuilder;
    use crate::{
        command::{CommandId, Request},
        state::{KeyType, Value},
    };
    use alloc::borrow::Cow;
    use arrayvec::ArrayVec;

    #[test]
    fn test_cmd_id() {
        let builder = RequestBuilder::new(CommandId::Stats);

        assert_eq!(
            builder.into_request(),
            Request {
                buf: [CommandId::Stats as u8].as_ref().into(),
                kind: CommandId::Stats,
                key_type: None,
                positions: Cow::Owned(ArrayVec::new()),
            }
        );
    }

    #[test]
    fn test_key_type() {
        let builder = RequestBuilder::new_with_key_type(CommandId::Decrement, KeyType::Integer);

        assert_eq!(
            builder.into_request(),
            Request {
                buf: [
                    0b1000_0000 | CommandId::Decrement as u8,
                    KeyType::Integer as u8,
                    0
                ]
                .as_ref()
                .into(),
                kind: CommandId::Decrement,
                key_type: Some(KeyType::Integer),
                positions: Cow::Owned(ArrayVec::new()),
            }
        );
    }

    #[test]
    fn test_arg_bytes() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Append, KeyType::List);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());

        let mut expected_args = Vec::new();
        expected_args.push(b"foo".as_ref());

        let mut positions = ArrayVec::new();
        positions.push(9);

        assert_eq!(
            builder.into_request(),
            Request {
                buf: [
                    0b1000_0000 | CommandId::Append as u8,
                    KeyType::List as u8,
                    1,
                    0,
                    0,
                    0,
                    3,
                    b'f',
                    b'o',
                    b'o',
                ]
                .as_ref()
                .into(),
                kind: CommandId::Append,
                key_type: Some(KeyType::List),
                positions: Cow::Owned(positions),
            }
        );
    }

    #[test]
    fn test_arg_value() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, KeyType::String);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.value(Value::Integer(123)).is_ok());

        let mut expected_args = Vec::new();
        expected_args.push(b"foo".as_ref());
        expected_args.push(123i64.to_be_bytes().as_ref());

        let mut positions = ArrayVec::new();
        positions.push(9);
        positions.push(21);

        assert_eq!(
            builder.into_request(),
            Request {
                buf: [
                    0b1000_0000 | CommandId::Set as u8,
                    KeyType::String as u8,
                    2,
                    0,
                    0,
                    0,
                    3,
                    b'f',
                    b'o',
                    b'o',
                    0,
                    0,
                    0,
                    8,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    123,
                ]
                .as_ref()
                .into(),
                kind: CommandId::Set,
                key_type: Some(KeyType::String),
                positions: Cow::Owned(positions),
            }
        );
    }

    #[test]
    fn test_positions() {
        let mut builder = RequestBuilder::new(CommandId::Decrement);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());

        assert_eq!(
            builder.buf.as_slice(),
            [CommandId::Decrement as u8, 1, 0, 0, 0, 3, b'f', b'o', b'o',]
        );
        assert_eq!(1, builder.positions.len());
        assert_eq!(Some(8), builder.positions.first().copied());

        let mut builder = RequestBuilder::new_with_key_type(CommandId::Get, KeyType::Boolean);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert_eq!(1, builder.positions.len());
        assert_eq!(Some(9), builder.positions.first().copied());
    }
}
