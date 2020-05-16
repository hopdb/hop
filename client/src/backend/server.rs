use super::Backend;
use async_trait::async_trait;
use hop_engine::{
    command::{
        request::ParseError,
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
        let mut cmd = vec![1, 1, 0, 0, 0, key.len() as u8];
        cmd.extend_from_slice(key);
        cmd.push(b'\n');

        let value = self.send_and_wait(&cmd).await?;

        match value {
            Value::Integer(int) => Ok(int),
            _ => Err(Error::BadResponse),
        }
    }

    async fn echo(&self, content: &[u8]) -> Result<Vec<u8>> {
        let mut cmd = vec![CommandId::Echo as u8, 1, 0, 0, 0, content.len() as u8];
        cmd.extend_from_slice(content);
        cmd.push(b'\n');

        let value = self.send_and_wait(&cmd).await?;

        match value {
            Value::Bytes(bytes) => Ok(bytes),
            _ => Err(Error::BadResponse),
        }
    }

    async fn increment(&self, key: &[u8], _: Option<KeyType>) -> Result<i64> {
        let mut cmd = vec![0, 1, 0, 0, 0, key.len() as u8];
        cmd.extend_from_slice(key);
        cmd.push(b'\n');

        let value = self.send_and_wait(&cmd).await?;

        match value {
            Value::Integer(int) => Ok(int),
            _ => Err(Error::BadResponse),
        }
    }
}
