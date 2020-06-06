use super::Backend;
use crate::model::StatsData;
use alloc::{boxed::Box, vec::Vec};
use async_trait::async_trait;
use core::{
    convert::TryInto,
    fmt::{Display, Formatter, Result as FmtResult},
    result::Result as StdResult,
};
use hop_engine::{
    command::{
        request::{ParseError, Request, RequestBuilder, RequestBuilderError},
        response::{Context, Instruction, Response},
        CommandId, DispatchError,
    },
    state::{KeyType, Value},
};
use std::{error::Error as StdError, io::Error as IoError};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream, ToSocketAddrs,
    },
    sync::Mutex,
};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    BadRequest { reason: ParseError },
    BadResponse,
    BuildingRequest { source: RequestBuilderError },
    Connecting { source: IoError },
    ConnectionClosed,
    Dispatching { reason: DispatchError },
    KeyTypeInvalid { number: u8 },
    KeyTypeUnsupported { key_type: KeyType },
    ReadingMessage { source: IoError },
    WritingMessage { source: IoError },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::BadRequest { reason } => {
                f.write_fmt(format_args!("server couldn't parse request: {:?}", reason))
            }
            Self::BadResponse => f.write_str("the response wasn't an expected type"),
            Self::BuildingRequest { source } => {
                f.write_fmt(format_args!("failed to build request: {:?}", source))
            }
            Self::Connecting { .. } => f.write_str("failed to connect"),
            Self::ConnectionClosed => f.write_str("connection closed"),
            Self::Dispatching { reason } => f.write_fmt(format_args!(
                "server couldn't process command: {:?}",
                reason
            )),
            Self::KeyTypeInvalid { number } => f.write_fmt(format_args!(
                "the provided key type ({}) is invalid",
                number
            )),
            Self::KeyTypeUnsupported { key_type } => f.write_fmt(format_args!(
                "key type {} is not supported by this command",
                *key_type as u8
            )),
            Self::ReadingMessage { .. } => f.write_str("failed to read a message"),
            Self::WritingMessage { .. } => f.write_str("failed to write a message"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::BadRequest { .. } => None,
            Self::BadResponse => None,
            Self::BuildingRequest { .. } => None,
            Self::Connecting { source } => Some(source),
            Self::ConnectionClosed => None,
            Self::Dispatching { .. } => None,
            Self::KeyTypeInvalid { .. } => None,
            Self::KeyTypeUnsupported { .. } => None,
            Self::ReadingMessage { source } => Some(source),
            Self::WritingMessage { source } => Some(source),
        }
    }
}

impl From<RequestBuilderError> for Error {
    fn from(source: RequestBuilderError) -> Self {
        Self::BuildingRequest { source }
    }
}

#[derive(Debug)]
pub struct ServerBackend {
    reader: Mutex<BufReader<OwnedReadHalf>>,
    writer: Mutex<OwnedWriteHalf>,
}

impl ServerBackend {
    pub async fn connect(addrs: impl ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(addrs)
            .await
            .map_err(|source| Error::Connecting { source })?;

        let (reader, writer) = stream.into_split();

        Ok(Self {
            reader: Mutex::new(BufReader::new(reader)),
            writer: Mutex::new(writer),
        })
    }

    async fn send_and_wait(&self, request: impl Into<Request<'_>>) -> Result<Value> {
        self.writer
            .lock()
            .await
            .write_all(request.into().as_bytes())
            .await
            .map_err(|source| Error::WritingMessage { source })?;

        let mut ctx = Context::new();
        let mut resp = Vec::with_capacity(1);

        let mut reader = self.reader.lock().await;

        loop {
            let read_amount = reader
                .read_exact(&mut resp)
                .await
                .map_err(|source| Error::ReadingMessage { source })?;

            if read_amount == 0 {
                return Err(Error::ConnectionClosed);
            }

            match ctx.feed(&resp).unwrap() {
                Instruction::Concluded(response) => {
                    return match response {
                        Response::Value(value) => Ok(value),
                        Response::DispatchError(reason) => Err(Error::Dispatching { reason }),
                        Response::ParseError(reason) => Err(Error::BadRequest { reason }),
                    }
                }
                Instruction::ReadBytes(bytes) => {
                    resp.reserve_exact(bytes);
                }
            }
        }
    }
}

#[async_trait]
impl Backend for ServerBackend {
    type Error = Error;

    async fn append<T: Into<Value> + Send>(&self, key: &[u8], value: T) -> Result<Value> {
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
            _ => return Err(Error::KeyTypeUnsupported { key_type }),
        }

        self.send_and_wait(builder).await
    }

    async fn decrement_by<T: Into<Value> + Send>(&self, key: &[u8], value: T) -> Result<Value> {
        let value = value.into();
        let key_type = value.kind();

        let mut builder = RequestBuilder::new_with_key_type(CommandId::DecrementBy, key_type);
        builder.bytes(key)?;

        if key_type != KeyType::Float && key_type != KeyType::Integer {
            return Err(Error::KeyTypeUnsupported { key_type });
        }

        builder.value(value)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Float(float) => Ok(Value::Float(float)),
            Value::Integer(int) => Ok(Value::Integer(int)),
            _ => Err(Error::BadResponse),
        }
    }

    async fn decrement(&self, key: &[u8], key_type: Option<KeyType>) -> Result<Value> {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Decrement, key_type);
        builder.bytes(key)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Float(float) => Ok(Value::Float(float)),
            Value::Integer(int) => Ok(Value::Integer(int)),
            _ => Err(Error::BadResponse),
        }
    }

    async fn delete(&self, key: &[u8]) -> Result<Vec<u8>> {
        let mut builder = RequestBuilder::new(CommandId::Delete);
        builder.bytes(key)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Bytes(bytes) => Ok(bytes),
            _ => Err(Error::BadResponse),
        }
    }

    async fn echo(&self, content: &[u8]) -> Result<Vec<Vec<u8>>> {
        let mut builder = RequestBuilder::new(CommandId::Echo);
        builder.bytes(content)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::List(args) => Ok(args),
            _ => Err(Error::BadResponse),
        }
    }

    async fn exists<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        keys: T,
    ) -> Result<bool> {
        let mut builder = RequestBuilder::new(CommandId::Exists);

        for key in keys {
            builder.bytes(key.as_ref())?;
        }

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Boolean(exists) => Ok(exists),
            _ => Err(Error::BadResponse),
        }
    }

    async fn get(&self, key: &[u8]) -> Result<Value> {
        let mut builder = RequestBuilder::new(CommandId::Get);
        builder.bytes(key)?;

        self.send_and_wait(builder).await
    }

    async fn increment_by<T: Into<Value> + Send>(&self, key: &[u8], value: T) -> Result<Value> {
        let value = value.into();
        let key_type = value.kind();

        if key_type != KeyType::Float && key_type != KeyType::Integer {
            return Err(Error::KeyTypeUnsupported { key_type });
        }

        let mut builder = RequestBuilder::new_with_key_type(CommandId::IncrementBy, key_type);
        builder.bytes(key)?;
        builder.value(value)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Float(float) => Ok(Value::Float(float)),
            Value::Integer(int) => Ok(Value::Integer(int)),
            _ => Err(Error::BadResponse),
        }
    }

    async fn increment(&self, key: &[u8], _: Option<KeyType>) -> Result<Value> {
        let mut builder = RequestBuilder::new(CommandId::Increment);
        builder.bytes(key)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Float(float) => Ok(Value::Float(float)),
            Value::Integer(int) => Ok(Value::Integer(int)),
            _ => Err(Error::BadResponse),
        }
    }

    async fn is<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        key_type: KeyType,
        keys: T,
    ) -> Result<bool> {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Is, key_type);

        for key in keys {
            builder.bytes(key.as_ref())?;
        }

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Boolean(exists) => Ok(exists),
            _ => Err(Error::BadResponse),
        }
    }

    async fn key_type(&self, key: &[u8]) -> Result<KeyType> {
        let mut builder = RequestBuilder::new(CommandId::Type);
        builder.bytes(key)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Integer(int) => {
                let number = int as u8;

                number
                    .try_into()
                    .map_err(|_| Error::KeyTypeInvalid { number })
            }
            _ => Err(Error::BadResponse),
        }
    }

    async fn keys(&self, key: &[u8]) -> Result<Vec<Vec<u8>>> {
        let mut builder = RequestBuilder::new(CommandId::Keys);
        builder.bytes(key)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::List(list) => Ok(list),
            _ => Err(Error::BadResponse),
        }
    }

    async fn length(&self, key: &[u8], key_type: Option<KeyType>) -> Result<i64> {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Length, key_type);
        builder.bytes(key)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Integer(int) => Ok(int),
            _ => Err(Error::BadResponse),
        }
    }

    async fn rename(&self, from: &[u8], to: &[u8]) -> Result<Vec<u8>> {
        let mut builder = RequestBuilder::new(CommandId::Rename);
        builder.bytes(from)?;
        builder.bytes(to)?;

        let value = self.send_and_wait(builder).await?;

        match value {
            Value::Bytes(bytes) => Ok(bytes),
            _ => Err(Error::BadResponse),
        }
    }

    async fn stats(&self) -> Result<StatsData> {
        let builder = RequestBuilder::new(CommandId::Stats);

        let value = self.send_and_wait(builder).await?;

        let map = match value {
            Value::Map(map) => map,
            _ => return Err(Error::BadResponse),
        };

        Ok(StatsData::new(map.into_iter().collect()))
    }

    async fn set<T: Into<Value> + Send>(&self, key: &[u8], value: T) -> Result<Value> {
        let value = value.into();
        let key_type = value.kind();

        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, key_type);
        builder.bytes(key)?;
        builder.value(value)?;

        self.send_and_wait(builder).await
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, ServerBackend};
    use core::fmt::Debug;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Error: Debug, Send, Sync);
    assert_impl_all!(ServerBackend: Debug, Send, Sync);
}
