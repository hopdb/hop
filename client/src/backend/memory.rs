use super::Backend;
use crate::model::StatsData;
use async_trait::async_trait;
use hop_engine::{
    command::{
        request,
        response::{Context, Instruction, Response},
        CommandId, DispatchError, Request,
    },
    state::{KeyType, Value},
    Hop,
};
use std::{
    convert::TryInto,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

fn request(id: CommandId, args: Option<Vec<Vec<u8>>>, kind: Option<KeyType>) -> Request {
    if let Some(kind) = kind {
        Request::new_with_type(id, args, kind)
    } else {
        Request::new(id, args)
    }
}

#[derive(Debug)]
pub enum Error {
    RunningCommand { source: DispatchError },
}

impl From<DispatchError> for Error {
    fn from(source: DispatchError) -> Self {
        Self::RunningCommand { source }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::RunningCommand { source } => f.write_fmt(format_args!("{}", source)),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::RunningCommand { .. } => None,
        }
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
}

#[async_trait]
impl Backend for MemoryBackend {
    type Error = Error;

    async fn decrement(&self, key: &[u8], kind: Option<KeyType>) -> Result<i64, Self::Error> {
        let req = request(CommandId::Decrement, Some(vec![key.to_vec()]), kind);
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let arr = resp.get(1..9).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    async fn delete(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let req = Request::new(CommandId::Delete, Some(vec![key.to_vec()]));
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        let bytes = match resp {
            Response::Value(Value::Bytes(bytes)) => bytes,
            other => panic!("Other response: {:?}", other),
        };

        Ok(bytes)
    }

    async fn echo(&self, content: &[u8]) -> Result<Vec<Vec<u8>>, Self::Error> {
        let req = Request::new(CommandId::Echo, Some(vec![content.to_vec()]));
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        match resp {
            Response::Value(Value::List(args)) => Ok(args),
            _ => panic!(),
        }
    }

    async fn exists<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        keys: T,
    ) -> Result<bool, Self::Error> {
        let args = keys
            .into_iter()
            .map(|arg| arg.as_ref().to_owned())
            .collect();
        let req = Request::new(CommandId::Exists, Some(args));
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        match resp {
            Response::Value(Value::Boolean(exists)) => Ok(exists),
            _ => panic!(),
        }
    }

    async fn increment(&self, key: &[u8], kind: Option<KeyType>) -> Result<i64, Self::Error> {
        let req = request(CommandId::Increment, Some(vec![key.to_vec()]), kind);
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        let int = match resp {
            Response::Value(Value::Integer(int)) => int,
            _ => panic!(),
        };

        Ok(int)
    }

    async fn is<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        key_type: KeyType,
        keys: T,
    ) -> Result<bool, Self::Error> {
        let args = keys
            .into_iter()
            .map(|arg| arg.as_ref().to_owned())
            .collect();
        let req = Request::new_with_type(CommandId::Is, Some(args), key_type);
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        match resp {
            Response::Value(Value::Boolean(exists)) => Ok(exists),
            _ => panic!(),
        }
    }

    async fn rename(&self, from: &[u8], to: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let req = Request::new(CommandId::Rename, Some(vec![from.to_vec(), to.to_vec()]));
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        let bytes = match resp {
            Response::Value(Value::Bytes(bytes)) => bytes,
            _ => panic!(),
        };

        Ok(bytes)
    }

    async fn set<T: Into<Value> + Send>(&self, key: &[u8], value: T) -> Result<Value, Self::Error> {
        let mut args = Vec::new();
        args.push(key.to_vec());

        let value = value.into();
        let key_type = value.kind();

        request::write_value_to_args(value, &mut args);

        let req = request(CommandId::Set, Some(args), Some(key_type));
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        match resp {
            Response::Value(value) => Ok(value),
            _ => panic!(),
        }
    }

    async fn stats(&self) -> Result<StatsData, Self::Error> {
        let req = request(CommandId::Stats, None, None);
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        let stats = match resp {
            Response::Value(Value::Map(stats)) => stats,
            _ => panic!(),
        };

        Ok(StatsData::new(stats.into_iter().collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::{Backend, MemoryBackend};
    use hop_engine::state::{
        object::{Boolean, Bytes, Float, Integer, Str},
        KeyType, Value,
    };

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
    async fn test_set_bool() {
        let backend = MemoryBackend::new();
        assert!(
            matches!(backend.set(b"foo", true).await, Ok(Value::Boolean(bool)) if bool == true)
        );
        assert!(matches!(
            backend.hop.state().typed_key::<Boolean>(b"foo").as_deref(),
            Some(true)
        ));
        assert!(
            matches!(backend.set(b"bar", false).await, Ok(Value::Boolean(bool)) if bool == false)
        );
        assert!(matches!(
            backend.hop.state().typed_key::<Boolean>(b"bar").as_deref(),
            Some(false)
        ));
    }

    #[tokio::test]
    async fn test_set_bytes() {
        let backend = MemoryBackend::new();
        assert!(
            matches!(backend.set(b"foo", [1u8, 2, 3].to_vec()).await, Ok(Value::Bytes(bytes)) if bytes == [1, 2, 3])
        );
        assert!(
            matches!(backend.hop.state().typed_key::<Bytes>(b"foo").as_deref(), Some(vec) if *vec == [1u8, 2, 3].to_vec())
        );
    }

    #[tokio::test]
    async fn test_set_float() {
        let backend = MemoryBackend::new();
        assert!(matches!(
            backend.set(b"foo", 1.23).await,
            Ok(Value::Float(_))
        ));
        assert!(backend.hop.state().typed_key::<Float>(b"foo").is_some());
    }

    #[tokio::test]
    async fn test_set_int() {
        let backend = MemoryBackend::new();
        assert!(matches!(
            backend.set(b"foo", 123).await,
            Ok(Value::Integer(123))
        ));
        assert!(matches!(
            backend.hop.state().typed_key::<Integer>(b"foo").as_deref(),
            Some(123)
        ));
    }

    #[tokio::test]
    async fn test_set_string() {
        let backend = MemoryBackend::new();
        assert!(
            matches!(backend.set(b"foo", "bar".to_owned()).await, Ok(Value::String(str)) if str == "bar")
        );
        assert!(
            matches!(backend.hop.state().typed_key::<Str>(b"foo").as_deref(), Some(s) if s == "bar")
        );
    }
}
