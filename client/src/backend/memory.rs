use super::Backend;
use crate::model::StatsData;
use alloc::{boxed::Box, vec::Vec};
use async_trait::async_trait;
use core::{
    convert::TryInto,
    fmt::{Display, Formatter, Result as FmtResult},
};
use hop_engine::{
    command::{
        request::{ParseError as RequestParseError, RequestBuilder, RequestBuilderError},
        response::{Context, Instruction, Response},
        CommandId, DispatchError, Request,
    },
    state::{KeyType, Value},
    Hop,
};

#[derive(Debug)]
pub enum Error {
    BadRequest { source: RequestParseError },
    BuildingRequest { source: RequestBuilderError },
    Dispatching { source: DispatchError },
    KeyTypeInvalid { number: u8 },
    KeyTypeUnsupported { key_type: KeyType, value: Value },
    RunningCommand { source: DispatchError },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::BadRequest { source } => {
                f.write_fmt(format_args!("request is invalid: {:?}", source))
            }
            Self::BuildingRequest { source } => {
                f.write_fmt(format_args!("failed to build request: {:?}", source))
            }
            Self::Dispatching { source } => {
                f.write_fmt(format_args!("dispatching the request failed: {:?}", source))
            }
            Self::KeyTypeInvalid { number } => f.write_fmt(format_args!(
                "the provided key type ({}) is invalid",
                number
            )),
            Self::KeyTypeUnsupported { key_type, value } => f.write_fmt(format_args!(
                "key type {} is not supported by this command (value: {:?})",
                *key_type as u8, value,
            )),
            Self::RunningCommand { source } => f.write_fmt(format_args!("{}", source)),
        }
    }
}

#[cfg(all(not(no_std), feature = "std"))]
mod if_std {
    use super::Error;
    use std::error::Error as StdError;

    impl StdError for Error {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            match self {
                Self::BadRequest { .. } => None,
                Self::BuildingRequest { .. } => None,
                Self::Dispatching { .. } => None,
                Self::KeyTypeInvalid { .. } => None,
                Self::KeyTypeUnsupported { .. } => None,
                Self::RunningCommand { .. } => None,
            }
        }
    }
}

impl From<DispatchError> for Error {
    fn from(source: DispatchError) -> Self {
        Self::RunningCommand { source }
    }
}

impl From<RequestBuilderError> for Error {
    fn from(source: RequestBuilderError) -> Self {
        Self::BuildingRequest { source }
    }
}

#[derive(Debug, Default)]
pub struct MemoryBackend {
    hop: Hop,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Default::default()
    }

    fn send<'a>(&self, req: impl Into<Request<'a>>) -> Result<Value, Error> {
        let mut resp = Vec::new();

        self.hop.dispatch(&req.into(), &mut resp)?;

        let mut ctx = Context::new();

        match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(Response::Value(value)) => Ok(value),
            Instruction::Concluded(Response::DispatchError(source)) => {
                Err(Error::Dispatching { source })
            }
            Instruction::Concluded(Response::ParseError(source)) => {
                Err(Error::BadRequest { source })
            }
            Instruction::ReadBytes(_) => unreachable!(),
        }
    }
}

#[async_trait]
impl Backend for MemoryBackend {
    type Error = Error;

    async fn append<T: Into<Value> + Send>(
        &self,
        key: &[u8],
        value: T,
    ) -> Result<Value, Self::Error> {
        let value = value.into();
        let key_type = value.kind();

        let mut builder = RequestBuilder::new_with_key_type(CommandId::Append, key_type);
        builder.bytes(key)?;

        match value {
            Value::Bytes(bytes) => {
                builder.bytes(bytes)?;
            }
            Value::List(list) => {
                for item in list {
                    builder.bytes(item)?;
                }
            }
            Value::String(string) => {
                builder.bytes(string.into_bytes())?;
            }
            value => return Err(Error::KeyTypeUnsupported { key_type, value }),
        }

        self.send(builder)
    }

    async fn decrement_by<T: Into<Value> + Send>(
        &self,
        key: &[u8],
        value: T,
    ) -> Result<Value, Self::Error> {
        let value = value.into();
        let key_type = value.kind();

        let mut builder = RequestBuilder::new_with_key_type(CommandId::DecrementBy, key_type);
        builder.bytes(key)?;

        if key_type != KeyType::Float && key_type != KeyType::Integer {
            return Err(Error::KeyTypeUnsupported { key_type, value });
        }

        builder.value(value)?;

        match self.send(builder)? {
            Value::Float(float) => Ok(Value::Float(float)),
            Value::Integer(int) => Ok(Value::Integer(int)),
            other => panic!("Other response: {:?}", other),
        }
    }

    async fn decrement(&self, key: &[u8], key_type: Option<KeyType>) -> Result<Value, Self::Error> {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Decrement, key_type);
        builder.bytes(key)?;

        match self.send(builder)? {
            Value::Float(float) => Ok(Value::Float(float)),
            Value::Integer(int) => Ok(Value::Integer(int)),
            other => panic!("Other response: {:?}", other),
        }
    }

    async fn delete(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let mut builder = RequestBuilder::new(CommandId::Delete);
        builder.bytes(key)?;

        match self.send(builder)? {
            Value::Bytes(bytes) => Ok(bytes),
            other => panic!("Other response: {:?}", other),
        }
    }

    async fn echo(&self, content: &[u8]) -> Result<Vec<Vec<u8>>, Self::Error> {
        let mut builder = RequestBuilder::new(CommandId::Echo);
        builder.bytes(content)?;

        match self.send(builder)? {
            Value::List(list) => Ok(list),
            _ => panic!(),
        }
    }

    async fn exists<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        keys: T,
    ) -> Result<bool, Self::Error> {
        let mut builder = RequestBuilder::new(CommandId::Exists);

        for key in keys {
            builder.bytes(key.as_ref())?;
        }

        match self.send(builder)? {
            Value::Boolean(exists) => Ok(exists),
            _ => panic!(),
        }
    }

    async fn get(&self, key: &[u8]) -> Result<Value, Self::Error> {
        let mut builder = RequestBuilder::new(CommandId::Get);
        builder.bytes(key)?;

        self.send(builder)
    }

    async fn increment_by<T: Into<Value> + Send>(
        &self,
        key: &[u8],
        value: T,
    ) -> Result<Value, Self::Error> {
        let value = value.into();
        let key_type = value.kind();

        let mut builder = RequestBuilder::new_with_key_type(CommandId::IncrementBy, key_type);
        builder.bytes(key)?;

        if key_type != KeyType::Float && key_type != KeyType::Integer {
            return Err(Error::KeyTypeUnsupported { key_type, value });
        }

        builder.value(value)?;

        match self.send(builder)? {
            Value::Float(float) => Ok(Value::Float(float)),
            Value::Integer(int) => Ok(Value::Integer(int)),
            other => panic!("Other response: {:?}", other),
        }
    }

    async fn increment(&self, key: &[u8], key_type: Option<KeyType>) -> Result<Value, Self::Error> {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Increment, key_type);
        builder.bytes(key)?;

        match self.send(builder)? {
            Value::Float(float) => Ok(Value::Float(float)),
            Value::Integer(int) => Ok(Value::Integer(int)),
            _ => panic!(),
        }
    }

    async fn is<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        key_type: KeyType,
        keys: T,
    ) -> Result<bool, Self::Error> {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Is, key_type);

        for key in keys {
            builder.bytes(key.as_ref())?;
        }

        match self.send(builder)? {
            Value::Boolean(exists) => Ok(exists),
            _ => panic!(),
        }
    }

    async fn key_type(&self, key: &[u8]) -> Result<KeyType, Self::Error> {
        let mut builder = RequestBuilder::new(CommandId::Type);
        builder.bytes(key)?;

        match self.send(builder)? {
            Value::Integer(int) => {
                let number = int as u8;

                number
                    .try_into()
                    .map_err(|_| Error::KeyTypeInvalid { number })
            }
            _ => panic!(),
        }
    }

    async fn keys(&self, key: &[u8]) -> Result<Vec<Vec<u8>>, Self::Error> {
        let mut builder = RequestBuilder::new(CommandId::Keys);
        builder.bytes(key)?;

        match self.send(builder)? {
            Value::List(list) => Ok(list),
            _ => panic!(),
        }
    }

    async fn length(&self, key: &[u8], key_type: Option<KeyType>) -> Result<i64, Self::Error> {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Length, key_type);
        builder.bytes(key)?;

        match self.send(builder)? {
            Value::Integer(int) => Ok(int),
            other => panic!("Other response: {:?}", other),
        }
    }

    async fn rename(&self, from: &[u8], to: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let mut builder = RequestBuilder::new(CommandId::Rename);
        builder.bytes(from)?;
        builder.bytes(to)?;

        match self.send(builder)? {
            Value::Bytes(bytes) => Ok(bytes),
            _ => panic!(),
        }
    }

    async fn set<T: Into<Value> + Send>(&self, key: &[u8], value: T) -> Result<Value, Self::Error> {
        let value = value.into();
        let key_type = value.kind();

        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, key_type);
        builder.bytes(key)?;
        builder.value(value)?;

        self.send(builder)
    }

    async fn stats(&self) -> Result<StatsData, Self::Error> {
        let builder = RequestBuilder::new(CommandId::Stats);

        let stats = match self.send(builder)? {
            Value::Map(stats) => stats,
            _ => panic!(),
        };

        Ok(StatsData::new(stats.into_iter().collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::{Backend, Error, MemoryBackend};
    use hop_engine::state::{KeyType, Value};
    use static_assertions::assert_impl_all;
    use std::fmt::Debug;

    assert_impl_all!(Error: Debug, Send, Sync);
    assert_impl_all!(MemoryBackend: Debug, Default, Send, Sync);

    #[tokio::test]
    async fn test_append() {
        let backend = MemoryBackend::new();

        assert!(backend
            .set(b"foo", Value::String("foo".to_owned()))
            .await
            .is_ok());
        assert!(backend
            .append(b"foo", Value::String("bar".to_owned()))
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_decrement() {
        let backend = MemoryBackend::new();

        assert!(matches!(
            backend.decrement(b"foo", None).await,
            Ok(Value::Integer(-1))
        ));
    }

    #[tokio::test]
    async fn test_echo() {
        let backend = MemoryBackend::new();
        assert!(matches!(backend.echo(b"test").await, Ok(vec) if vec == vec![b"test"]));
    }

    #[tokio::test]
    async fn test_is() {
        let backend = MemoryBackend::new();
        backend.set(b"foo", Value::Boolean(true)).await.unwrap();

        assert!(backend.is(KeyType::Boolean, &["foo"]).await.unwrap());
        assert!(!backend.is(KeyType::Integer, &["foo"]).await.unwrap());
    }

    #[tokio::test]
    async fn test_length() {
        let backend = MemoryBackend::new();
        assert!(backend
            .set(b"foo", Value::String("foo".to_owned()))
            .await
            .is_ok());
        assert_eq!(
            3,
            backend.length(b"foo", Some(KeyType::String)).await.unwrap()
        );
    }

    #[tokio::test]
    async fn test_set_bool() {
        let backend = MemoryBackend::new();
        assert!(
            matches!(backend.set(b"foo", true).await, Ok(Value::Boolean(bool)) if bool == true)
        );
        assert_eq!(
            Some(&true),
            backend
                .hop
                .state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_boolean_ref),
        );
        assert!(
            matches!(backend.set(b"bar", false).await, Ok(Value::Boolean(bool)) if bool == false)
        );
        assert_eq!(
            Some(&false),
            backend
                .hop
                .state()
                .key_ref(b"bar")
                .as_deref()
                .and_then(Value::as_boolean_ref),
        );
    }

    #[tokio::test]
    async fn test_set_bytes() {
        let backend = MemoryBackend::new();
        assert!(
            matches!(backend.set(b"foo", [1u8, 2, 3].to_vec()).await, Ok(Value::Bytes(bytes)) if bytes == [1, 2, 3])
        );
        assert_eq!(
            Some([1u8, 2, 3].as_ref()),
            backend
                .hop
                .state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_bytes_ref),
        );
    }

    #[tokio::test]
    async fn test_set_float() {
        let backend = MemoryBackend::new();
        assert!(matches!(
            backend.set(b"foo", 1.23).await,
            Ok(Value::Float(_))
        ));
        assert!(backend
            .hop
            .state()
            .key_ref(b"foo")
            .as_deref()
            .and_then(Value::as_float_ref)
            .is_some());
    }

    #[tokio::test]
    async fn test_set_int() {
        let backend = MemoryBackend::new();
        assert!(matches!(
            backend.set(b"foo", 123).await,
            Ok(Value::Integer(123))
        ));
        assert!(matches!(
            backend
                .hop
                .state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_integer_ref),
            Some(123)
        ));
    }

    #[tokio::test]
    async fn test_set_string() {
        let backend = MemoryBackend::new();
        assert!(
            matches!(backend.set(b"foo", "bar".to_owned()).await, Ok(Value::String(str)) if str == "bar")
        );
        assert_eq!(
            Some("bar"),
            backend
                .hop
                .state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_string_ref),
        );
    }
}
