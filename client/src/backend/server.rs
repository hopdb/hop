use super::Backend;
use crate::model::StatsData;
use async_trait::async_trait;
use hop_engine::{
    command::{
        request::{self, ParseError, Request},
        response::{Context, Instruction, Response},
        CommandId, DispatchError,
    },
    state::{KeyType, Value},
};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    result::Result as StdResult,
};
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
    Connecting { source: IoError },
    ConnectionClosed,
    Dispatching { reason: DispatchError },
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
            Self::Connecting { .. } => f.write_str("failed to connect"),
            Self::ConnectionClosed => f.write_str("connection closed"),
            Self::Dispatching { reason } => f.write_fmt(format_args!(
                "server couldn't process command: {:?}",
                reason
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
            Self::Connecting { source } => Some(source),
            Self::ConnectionClosed => None,
            Self::Dispatching { .. } => None,
            Self::ReadingMessage { source } => Some(source),
            Self::WritingMessage { source } => Some(source),
        }
    }
}

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

    async fn send_and_wait(&self, send: &[u8]) -> Result<Value> {
        self.writer
            .lock()
            .await
            .write_all(send)
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

    async fn decrement(&self, key: &[u8], _: Option<KeyType>) -> Result<i64> {
        let mut args = Vec::with_capacity(1);
        args.push(key.to_vec());
        let req = Request::new(CommandId::Decrement, Some(args));

        let value = self.send_and_wait(&req.into_bytes()).await?;

        match value {
            Value::Integer(int) => Ok(int),
            _ => Err(Error::BadResponse),
        }
    }

    async fn delete(&self, key: &[u8]) -> Result<Vec<u8>> {
        let mut args = Vec::with_capacity(1);
        args.push(key.to_vec());

        let req = Request::new(CommandId::Delete, Some(args));

        let value = self.send_and_wait(&req.into_bytes()).await?;

        match value {
            Value::Bytes(bytes) => Ok(bytes),
            _ => Err(Error::BadResponse),
        }
    }

    async fn echo(&self, content: &[u8]) -> Result<Vec<Vec<u8>>> {
        let mut args = Vec::with_capacity(1);
        args.push(content.to_vec());
        let req = Request::new(CommandId::Echo, Some(args));

        let value = self.send_and_wait(&req.into_bytes()).await?;

        match value {
            Value::List(args) => Ok(args),
            _ => Err(Error::BadResponse),
        }
    }

    async fn exists<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        keys: T,
    ) -> Result<bool> {
        let args = keys
            .into_iter()
            .map(|key| key.as_ref().to_owned())
            .collect();
        let req = Request::new(CommandId::Exists, Some(args));

        let value = self.send_and_wait(&req.into_bytes()).await?;

        match value {
            Value::Boolean(exists) => Ok(exists),
            _ => Err(Error::BadResponse),
        }
    }

    async fn increment(&self, key: &[u8], _: Option<KeyType>) -> Result<i64> {
        let mut args = Vec::with_capacity(1);
        args.push(key.to_vec());
        let req = Request::new(CommandId::Increment, Some(args));

        let value = self.send_and_wait(&req.into_bytes()).await?;

        match value {
            Value::Integer(int) => Ok(int),
            _ => Err(Error::BadResponse),
        }
    }

    async fn is<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        key_type: KeyType,
        keys: T,
    ) -> Result<bool> {
        let args = keys
            .into_iter()
            .map(|key| key.as_ref().to_owned())
            .collect();
        let req = Request::new_with_type(CommandId::Is, Some(args), key_type);

        let value = self.send_and_wait(&req.into_bytes()).await?;

        match value {
            Value::Boolean(exists) => Ok(exists),
            _ => Err(Error::BadResponse),
        }
    }

    async fn keys(&self, key: &[u8]) -> Result<Vec<Vec<u8>>> {
        let mut args = Vec::with_capacity(1);
        args.push(key.to_vec());

        let req = Request::new(CommandId::Keys, Some(args));

        let value = self.send_and_wait(&req.into_bytes()).await?;

        match value {
            Value::List(list) => Ok(list),
            _ => Err(Error::BadResponse),
        }
    }

    async fn rename(&self, from: &[u8], to: &[u8]) -> Result<Vec<u8>> {
        let mut args = Vec::with_capacity(2);
        args.push(from.to_vec());
        args.push(to.to_vec());

        let req = Request::new(CommandId::Rename, Some(args));

        let value = self.send_and_wait(&req.into_bytes()).await?;

        match value {
            Value::Bytes(bytes) => Ok(bytes),
            _ => Err(Error::BadResponse),
        }
    }

    async fn stats(&self) -> Result<StatsData> {
        let req = Request::new(CommandId::Stats, None);

        let value = self.send_and_wait(&req.into_bytes()).await?;

        let map = match value {
            Value::Map(map) => map,
            _ => return Err(Error::BadResponse),
        };

        Ok(StatsData::new(map.into_iter().collect()))
    }

    async fn set<T: Into<Value> + Send>(&self, key: &[u8], value: T) -> Result<Value> {
        let mut args = Vec::new();
        args.push(key.to_vec());

        let value = value.into();
        let key_type = value.kind();

        request::write_value_to_args(value, &mut args);

        let req = Request::new_with_type(CommandId::Set, Some(args), key_type);

        self.send_and_wait(&req.into_bytes()).await
    }
}
