use super::Backend;
use async_std::{
    io::BufReader,
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
    sync::Mutex,
};
use async_trait::async_trait;
use hop_lib::command::CommandId;
use std::{
    convert::TryInto,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    result::Result as StdResult,
};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Connecting { source: IoError },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Connecting { .. } => f.write_str("failed to connect"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Connecting { source } => Some(source),
        }
    }
}

pub struct ServerBackend {
    stream: Mutex<TcpStream>,
}

impl ServerBackend {
    pub async fn connect(addrs: impl ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(addrs)
            .await
            .map_err(|source| Error::Connecting { source })?;

        Ok(Self {
            stream: Mutex::new(stream),
        })
    }

    async fn send_and_wait(&self, send: Vec<u8>) -> Result<Vec<u8>> {
        let mut reader = {
            let mut stream = self.stream.lock().await;
            stream.write_all(&send).await.unwrap();

            BufReader::new(stream.clone())
        };

        let mut s = Vec::new();
        reader.read_until(b'\n', &mut s).await.unwrap();

        Ok(s)
    }
}

#[async_trait]
impl Backend for ServerBackend {
    type Error = Error;

    async fn decrement(&self, key: &[u8]) -> Result<i64> {
        let mut cmd = vec![1, 1, 0, 0, 0, key.len() as u8];
        cmd.extend_from_slice(key);
        cmd.push(b'\n');

        let s = self.send_and_wait(cmd).await?;

        let arr = s.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    async fn echo(&self, content: &[u8]) -> Result<Vec<u8>> {
        let mut cmd = vec![CommandId::Echo as u8, 1, 0, 0, 0, content.len() as u8];
        cmd.extend_from_slice(content);
        cmd.push(b'\n');

        self.send_and_wait(cmd).await
    }

    async fn increment(&self, key: &[u8]) -> Result<i64> {
        let mut cmd = vec![0, 1, 0, 0, 0, key.len() as u8];
        cmd.extend_from_slice(key);
        cmd.push(b'\n');

        let s = self.send_and_wait(cmd).await?;

        let arr = s.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }
}
