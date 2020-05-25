use super::Request;
use crate::{
    command::CommandId,
    state::{KeyType, Value},
};
use alloc::vec::Vec;
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
    arguments: Option<Vec<Vec<u8>>>,
    cmd_id: CommandId,
    key_type: Option<KeyType>,
}

impl RequestBuilder {
    /// Create a new request builder.
    pub fn new(cmd_id: CommandId) -> Self {
        Self {
            arguments: None,
            cmd_id,
            key_type: None,
        }
    }

    /// Consume the builder and return the request serialised as bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.into_request().into_bytes()
    }

    /// Consume the builder and return the built request.
    pub fn into_request(self) -> Request {
        Request {
            args: self.arguments,
            key_type: self.key_type,
            kind: self.cmd_id,
        }
    }

    /// Retrieve an immutable reference to the command ID.
    pub fn command_id_ref(&self) -> &CommandId {
        &self.cmd_id
    }

    /// Set the key type.
    pub fn key_type(&mut self, key_type: KeyType) -> &mut Self {
        self.key_type.replace(key_type);

        self
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
                let mut buf = Vec::with_capacity(1);
                buf.push(bool as u8);

                self.push_argument(buf)?;
            }
            Value::Bytes(bytes) => {
                if bytes.is_empty() {
                    return Err(RequestBuilderError::ValueEmpty);
                }

                self.push_argument(bytes)?;
            }
            Value::Float(float) => self.push_argument(float.to_be_bytes().to_vec())?,
            Value::Integer(int) => self.push_argument(int.to_be_bytes().to_vec())?,
            Value::List(list) => {
                if self.arguments_would_overfill(list.len()) {
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
                if self.arguments_would_overfill(map.len()) {
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
                if self.arguments_would_overfill(set.len()) {
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

                self.push_argument(string.into_bytes())?;
            }
        }

        Ok(self)
    }

    /// Retrieve an immutable reference to an argument.
    pub fn argument_ref(&self, idx: usize) -> Option<&[u8]> {
        let arg = self.arguments.as_ref()?.get(idx)?;

        Some(arg.as_slice())
    }

    fn arguments_would_overfill(&self, len: usize) -> bool {
        match self.arguments.as_ref() {
            Some(arguments) => arguments.len() + len > 255,
            None => len > 255,
        }
    }

    /// Pushes an argument to the list.
    ///
    /// # Errors
    ///
    /// Returns [`RequestBuilderError::TooManyArguments`] if the list of
    /// arguments is already full.
    ///
    /// [`RequestBuilderError::TooManyArguments`]: enum.RequestBuilderError.html#variant.TooManyArguments
    fn push_argument(&mut self, argument: Vec<u8>) -> Result<(), RequestBuilderError> {
        if let Some(arguments) = self.arguments.as_mut() {
            if arguments.len() >= 255 {
                return Err(RequestBuilderError::TooManyArguments);
            }

            arguments.push(argument);
        } else {
            let mut arguments = Vec::new();
            arguments.push(argument);

            self.arguments.replace(arguments);
        }

        Ok(())
    }
}

impl From<Request> for RequestBuilder {
    fn from(request: Request) -> Self {
        let mut builder = Self::new(request.kind);

        if let Some(key_type) = request.key_type {
            builder.key_type(key_type);
        }

        if let Some(args) = request.args {
            builder.arguments.replace(args);
        }

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

    #[test]
    fn test_cmd_id() {
        let builder = RequestBuilder::new(CommandId::Stats);

        assert_eq!(
            builder.into_request(),
            Request {
                args: None,
                kind: CommandId::Stats,
                key_type: None,
            }
        );
    }

    #[test]
    fn test_key_type() {
        let mut builder = RequestBuilder::new(CommandId::Decrement);
        builder.key_type(KeyType::Integer);

        assert_eq!(
            builder.into_request(),
            Request {
                args: None,
                kind: CommandId::Decrement,
                key_type: Some(KeyType::Integer),
            }
        );
    }

    #[test]
    fn test_arg_bytes() {
        let mut builder = RequestBuilder::new(CommandId::Append);
        builder.key_type(KeyType::List);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());

        let mut expected_args = Vec::new();
        expected_args.push(b"foo".to_vec());

        assert_eq!(
            builder.into_request(),
            Request {
                args: Some(expected_args),
                kind: CommandId::Append,
                key_type: Some(KeyType::List),
            }
        );
    }

    #[test]
    fn test_arg_value() {
        let mut builder = RequestBuilder::new(CommandId::Set);
        builder.key_type(KeyType::String);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.value(Value::Integer(123)).is_ok());

        let mut expected_args = Vec::new();
        expected_args.push(b"foo".to_vec());
        expected_args.push(123i64.to_be_bytes().to_vec());

        assert_eq!(
            builder.into_request(),
            Request {
                args: Some(expected_args),
                kind: CommandId::Set,
                key_type: Some(KeyType::String),
            }
        );
    }
}
